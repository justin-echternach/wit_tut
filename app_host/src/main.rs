use std::env;

use wasmtime::{
    component::{Component, Linker, ResourceTable},
    Config, Engine, Store
};
use wasmtime_wasi_http::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

//use wasmtime::component::bindgen;

//bindgen!(in "../web/wit");

struct AppHost {
    table: ResourceTable,
    ctx: WasiCtx,
    http: WasiHttpCtx,
}
impl WasiView for AppHost {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}
impl WasiHttpView for AppHost {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <path_to_wasm_component> <function_name>", args[0]);
        return Err("Insufficient arguments provided".into());
    }

    let comp_file_path = &args[1];
    let func_name = &args[2];
    let wasm_args = &args[3..];

    run(comp_file_path, func_name, wasm_args).await
}

async fn run(comp_file_path: &str, func_name: &str, wasm_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {    
    let engine = setup_wasmtime_engine()?;
    
    let component = load_component(&engine, &comp_file_path).await?;
    
    execute_component(&engine, &component, &func_name, wasm_args).await?;

    Ok(())
}

fn setup_host() -> Result<AppHost, Box<dyn std::error::Error>> {
    let host = AppHost {
        table: ResourceTable::new(),
        ctx: WasiCtxBuilder::new()
            //.allow_blocking_current_thread(true)
            .inherit_stdio()
            .inherit_args()
            .inherit_env()
            .allow_ip_name_lookup(true)
            .allow_tcp(true)
            .allow_udp(true)
            .inherit_network()
            .build(),
        http: WasiHttpCtx::new(),
    };
    Ok(host)
}

fn setup_wasmtime_engine() -> Result<Engine, Box<dyn std::error::Error>> {
    let mut config = Config::default();
    config.wasm_component_model(true);
    config.async_support(true);

    

    Ok(Engine::new(&config)?)
}

async fn load_component(engine: &Engine, comp_file_path: &str) -> Result<Component, Box<dyn std::error::Error>> {
    Ok(Component::from_file(engine, comp_file_path)?)
}

async fn execute_component(engine: &Engine, component: &Component, func_name: &str, wasm_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut store: Store<AppHost> = Store::new(&engine, setup_host()?);
    let mut linker: Linker<AppHost> = Linker::new(&engine);
    
    wasmtime_wasi::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::proxy::add_only_http_to_linker(&mut linker)?;
    
    let binding = component.component_type();
    let mut exports = binding.exports(&engine);
    
    let export_name = exports.next().unwrap().0; // grab first export name (e.g. "tut:web/iweb@0.1.0")
    for (comp_name, comp_item) in binding.exports(&engine) {
        println!("Export name: {}, kind: {:?}", comp_name, comp_item);
    }

    let instance = linker.instantiate_async(&mut store, &component).await?;
    //let instance = linker.instantiate(&mut store, &component)?;    
    
    let func = {
        let mut exports = instance.exports(&mut store);
        if let Some(mut export_inst) = exports.instance(&export_name) {//"tut:web/iweb@0.1.0") {
            export_inst.func(&func_name).expect("Function not found")
        } else {
            return Err(format!("Instance export not found: {}", export_name).into());
        }
    };

    let func_params = func.params(&mut store);
    let func_results = func.results(&mut store);

    println!("{:?}", func_params);   
    println!("{:?}", func_results);

    let params = get_params(&func_params, &wasm_args)?;
    let mut results = get_results(&func_results)?;
    let result = func.call_async(&mut store, &params[..], &mut results[..]).await?;
    func.post_return_async(store).await?;

    println!("{:?}", result);
    println!("{:?}", results);    

    // with typed
    //let typed = func.typed::<(), (String,)>(&store)?;
    //let result = typed.call_async(&mut store, ()).await?;
    //println!("{:?}", result);

    //// with bindgen
    //let bindings = Web::new(&mut store, &instance)?;    
    ////let (bindings, _) = Web::instantiate(&mut store, &component, &linker)?;
    // let result = bindings.tut_web_iweb().call_hello_world(&mut store)?;
    // println!("{:?}", result);

    Ok(())
}

fn get_params(func_params: &[wasmtime::component::Type], wasm_args: &[String]) -> Result<Vec<wasmtime::component::Val>, Box<dyn std::error::Error>> {
    let mut params = Vec::new();
    for (i, param) in func_params.iter().enumerate() {
        match param {
            wasmtime::component::Type::String => {
                params.push(wasmtime::component::Val::String(wasm_args[i].clone()));
            },
            wasmtime::component::Type::Enum(_) => {
                // Assuming the Enum is a string representation in wasm_args
                params.push(wasmtime::component::Val::Enum(wasm_args[i].clone()));
            },
            wasmtime::component::Type::Bool => {
                let arg = wasm_args[i].parse::<bool>().map_err(|_| "Argument must be a valid bool")?;
                params.push(wasmtime::component::Val::Bool(arg));
            },
            wasmtime::component::Type::U8 => {
                let arg = wasm_args[i].parse::<u8>().map_err(|_| "Argument must be a valid u8")?;
                params.push(wasmtime::component::Val::U8(arg));
            },
            wasmtime::component::Type::U16 => {
                let arg = wasm_args[i].parse::<u16>().map_err(|_| "Argument must be a valid u16")?;
                params.push(wasmtime::component::Val::U16(arg));
            },
            wasmtime::component::Type::U32 => {
                let arg = wasm_args[i].parse::<u32>().map_err(|_| "Argument must be a valid u32")?;
                params.push(wasmtime::component::Val::U32(arg));
            },
            wasmtime::component::Type::U64 => {
                let arg = wasm_args[i].parse::<u64>().map_err(|_| "Argument must be a valid u64")?;
                params.push(wasmtime::component::Val::U64(arg));
            },
            wasmtime::component::Type::S8 => {
                let arg = wasm_args[i].parse::<i8>().map_err(|_| "Argument must be a valid i8")?;
                params.push(wasmtime::component::Val::S8(arg));
            },
            wasmtime::component::Type::S16 => {
                let arg = wasm_args[i].parse::<i16>().map_err(|_| "Argument must be a valid i16")?;
                params.push(wasmtime::component::Val::S16(arg));
            },
            wasmtime::component::Type::S32 => {
                let arg = wasm_args[i].parse::<i32>().map_err(|_| "Argument must be a valid i32")?;
                params.push(wasmtime::component::Val::S32(arg));
            },           
            wasmtime::component::Type::S64 => {
                let arg = wasm_args[i].parse::<i64>().map_err(|_| "Argument must be a valid i64")?;
                params.push(wasmtime::component::Val::S64(arg));
            },
            wasmtime::component::Type::Float32 => {
                let arg = wasm_args[i].parse::<f32>().map_err(|_| "Argument must be a valid f32")?;
                params.push(wasmtime::component::Val::Float32(arg));
            },
            wasmtime::component::Type::Float64 => {
                let arg = wasm_args[i].parse::<f64>().map_err(|_| "Argument must be a valid f64")?;
                params.push(wasmtime::component::Val::Float64(arg));
            },
            _ => return Err(format!("Unsupported parameter type: {:?}", param).into()),
        }
    }

    Ok(params)
}

fn get_results(func_results: &[wasmtime::component::Type]) -> Result<Vec<wasmtime::component::Val>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    for (_i, result) in func_results.iter().enumerate() {
        match result {
            wasmtime::component::Type::String => results.push(wasmtime::component::Val::String("".to_string())),
            wasmtime::component::Type::Bool => results.push(wasmtime::component::Val::Bool(false)),
            wasmtime::component::Type::Enum(_) => results.push(wasmtime::component::Val::Enum("".to_string())),
            wasmtime::component::Type::U8 => results.push(wasmtime::component::Val::U8(0)),
            wasmtime::component::Type::U16 => results.push(wasmtime::component::Val::U16(0)),
            wasmtime::component::Type::U32 => results.push(wasmtime::component::Val::U32(0)),
            wasmtime::component::Type::U64 => results.push(wasmtime::component::Val::U64(0)),
            wasmtime::component::Type::S8 => results.push(wasmtime::component::Val::S8(0)),
            wasmtime::component::Type::S16 => results.push(wasmtime::component::Val::S16(0)),
            wasmtime::component::Type::S32 => results.push(wasmtime::component::Val::S32(0)),
            wasmtime::component::Type::S64 => results.push(wasmtime::component::Val::S64(0)),
            wasmtime::component::Type::Float32 => results.push(wasmtime::component::Val::Float32(0.0)),
            wasmtime::component::Type::Float64 => results.push(wasmtime::component::Val::Float64(0.0)),
            wasmtime::component::Type::List(_) => results.push(wasmtime::component::Val::List(Vec::new())),
            _ => return Err(format!("Unsupported result type: {:?}", result).into()),
        }
    }

    Ok(results)
}


