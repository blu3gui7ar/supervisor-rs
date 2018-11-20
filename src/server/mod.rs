use super::Config;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process::{Child, Command};
use yaml_rust::{ScanError, Yaml, YamlEmitter, YamlLoader};

/*/tmp/server.yml
Loadpath:
  - /tmp/client/
*/

struct ServerConfig {
    load_path: String,
}

impl ServerConfig {
    fn load(filename: &str) -> Result<Self, String> {
        let contents = File::open(filename);
        let mut string_result = String::new();
        match contents {
            Ok(mut cont) => {
                cont.read_to_string(&mut string_result);
                return Self::read_from_str(string_result.as_str());
            }
            Err(e) => return Err(e.description().to_string()),
        }
    }
    fn read_from_str(input: &str) -> Result<Self, String> {
        let temp = YamlLoader::load_from_str(input);
        let mut result: Self;

        match temp {
            Ok(docs) => {
                let doc = &docs[0];
                result = ServerConfig {
                    load_path: doc["Loadpath"][0].clone().into_string().unwrap(),
                }
            }
            Err(e) => return Err(e.description().to_string()),
        }

        Ok(result)
    }
}

pub fn start_new_child(config: &mut Config) -> io::Result<Child> {
    let (com, args) = config.split_args();

    let mut command = Command::new(&com);

    match args {
        Some(arg) => {
            command.args(arg.split(' ').collect::<Vec<&str>>());
        }
        _ => (),
    };

    //run command and give child handle
    let child = command
        .stdout(File::create(config.stdout.as_ref().unwrap()).unwrap())
        .spawn();

    match child {
        Ok(ref c) => {
            config.child_id = Some(c.id());
            return child;
        }
        _ => return child,
    };

    child
}

//Receive server config and start a new server
//new server including:
//1. a way receive command from client
//2. first start will start all children in config path
//3. then keep listening commands and can restart each of them
pub fn start_new_server() {
    let mut server_conf = ServerConfig::load("/tmp/server.yml");
}

struct Kindergarten {
    name_list: HashMap<u32, Config>,
}

impl Kindergarten {
    pub fn new() -> Self {
        Kindergarten {
            name_list: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: u32, conf: Config) {
        self.name_list.insert(id, conf);
    }
}

//:= MARK: log: store children ids
//check if child still running
//when restart, check ids in log. if id proceesing exsit, means supervisor dead accidently
pub fn day_care() {
    //sleep_ms(2000);
    println!("{:?}", "hahah");
}
