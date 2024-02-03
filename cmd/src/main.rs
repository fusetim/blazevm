use std::{path::Path, process::exit};

use clap::Parser;
use pretty_env_logger::env_logger::{Builder, Env};
use reader::descriptor::{self, ClassName, MethodDescriptor};
use vm::{
    class_loader::{ClassLoader, ClassPathDirEntry},
    class_manager::LoadedClass,
    Vm,
};

const MAIN_METHOD_DESCRIPTOR: MethodDescriptor = MethodDescriptor {
    return_type: None,
    parameters: vec![],
};

#[derive(Parser, Debug)]
#[clap(name = "blazevm-cli", version, author, about)]
pub struct Opts {
    /// The classpath to use
    #[clap(short, long, default_value = "./classpath")]
    pub classpath: Vec<String>,

    /// The class to run
    #[clap(value_parser=parse_main_class, required = true)]
    pub main_class: ClassName,
}

fn parse_main_class(input: &str) -> Result<ClassName, descriptor::DescriptorError> {
    descriptor::parse_class_name(input.trim())
}

fn main() {
    pretty_env_logger::formatted_builder()
        .parse_env(Env::default().default_filter_or("info,vm=trace,reader=trace"))
        .init();
    let opts: Opts = Opts::parse();
    log::info!("BlazeVM starting up...");
    let mut class_loader = ClassLoader::new();
    for classpath in opts.classpath.iter() {
        log::info!("Adding classpath: {}", classpath);
        let class_path = ClassPathDirEntry::new(classpath);
        class_loader.add_class_path_entry(Box::new(class_path));
    }
    log::info!("Loading Main class: {}", opts.main_class);
    let mut vm = Vm::new(class_loader);
    let main_name: String = opts.main_class.as_binary_name();
    let thread_id = match vm.class_manager_mut().get_or_resolve_class(&main_name) {
        Ok(main_class) => {
            log::info!("Main class loaded: {:?}", main_class.id());
            let LoadedClass::Loaded(main_class) = main_class else {
                log::error!(
                    "Main class is not correctly initialized: {:?}",
                    main_class.id()
                );
                exit(-1);
            };
            let class_id = main_class.id;
            let Some((main_method, _)) = main_class.get_method("main", &MAIN_METHOD_DESCRIPTOR)
            else {
                log::error!("Main method not found in class: {:?}", &main_class.id);
                exit(-2);
            };
            log::info!("Main method loaded.");
            let args = vec![];
            vm.create_thread(&class_id, main_method, args)
        }
        Err(e) => {
            log::error!("Error loading main class, cause:\n{}", e);
            exit(-1);
        }
    };
    log::info!("Starting main thread: {}", thread_id);
    match vm.execute_thread(thread_id) {
        Ok(()) => log::info!("Main thread finished."),
        Err(e) => log::error!("Main thread failed: {}", e),
    }
    log::info!("BlazeVM shutting down...");
    exit(0);
}
