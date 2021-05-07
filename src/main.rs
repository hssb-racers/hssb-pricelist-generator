use yaml_rust::{Yaml, YamlLoader};
use glob::glob;
use clap::{AppSettings, Clap};
use log::{debug, LevelFilter};
use simple_logger::SimpleLogger;
use std::path::{Path, PathBuf};
use std::fs;
use std::fmt;

#[derive(Clap)]
#[clap(version = "0.1", author = "Sariya Melody <sariya@sariya.garden>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Folder which contains salvage data assets
    salvage_data_path: String,
    /// A level of verbosity, and can be used multiple times.
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}


#[derive(Debug)]
struct SalvageRewardData {
    name: String,
    min_initial_value: f64,
    max_initial_value: f64,
    mass_based_value: bool,
}

impl fmt::Display for SalvageRewardData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut pretty: String;
        // preferred data format:
        // NAME: XX / ea
        // NAME: XX - YY / ea
        // NAME: XX / kg
        // NAME: xx - YY / kg
        pretty = self.name.clone() + ": ";

        if self.min_initial_value != self.max_initial_value {
            pretty = pretty + format!("{} - {} / ", self.min_initial_value, self.max_initial_value).as_str();
        } else {
            pretty = pretty + format!("{} / ", self.min_initial_value).as_str();
        }
        pretty = pretty + format!("{}", if self.mass_based_value {"kg"} else {"ea"}).as_str();

        write!(f, "{}", pretty)
    }
}

fn convert_file_to_salvage_reward_data(ref path: PathBuf) -> SalvageRewardData {
    debug!("parsing {}", path.display());
    let yaml = YamlLoader::load_from_str(fs::read_to_string(path).expect(format!("failed to parse file {}", path.display()).as_str()).as_str()).unwrap();
    let doc = &yaml[0];
    let mono_behaviour = &doc["MonoBehaviour"];
    let name = mono_behaviour["m_Name"].as_str().unwrap().into();
    let data = &mono_behaviour["m_Data"];
    let awarded_currencies = &data["m_AwardedCurrencies"];
    let (mut min_initial_value, mut max_initial_value, mut mass_based_value) = (0.0, 0.0, false);
    if awarded_currencies.is_array() && awarded_currencies.as_vec().unwrap().len() != 0 {
        debug!("{:#?}", awarded_currencies);
        min_initial_value = match awarded_currencies[0]["m_MinInitialValue"] {
            Yaml::Real(_) => awarded_currencies[0]["m_MinInitialValue"].as_f64().unwrap(),
            Yaml::Integer(i) => i as f64,
            _ => 0.into()
        };
        max_initial_value = match awarded_currencies[0]["m_MaxInitialValue"] {
            Yaml::Real(_) => awarded_currencies[0]["m_MaxInitialValue"].as_f64().unwrap(),
            Yaml::Integer(i) => i as f64,
            _ => 0.into()
        };
        mass_based_value = awarded_currencies[0]["m_MassBasedValue"].as_i64().unwrap() == 1;
    }

    SalvageRewardData{
        name,
        min_initial_value,
        max_initial_value,
        mass_based_value
    }
}

fn main() {
    let opts = Opts::parse();
    SimpleLogger::new()
        .with_level(match opts.verbose {
            0 => LevelFilter::Error,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            3 | _ => LevelFilter::Trace,
        })
        .init()
        .unwrap();

    let mut salvage_data = Vec::new();
    for entry in glob(Path::new(&opts.salvage_data_path).join("SALV_*.asset").to_str().unwrap()).expect("failed to parse glob string") {
        if let Ok(path) = entry {
            salvage_data.push(convert_file_to_salvage_reward_data(path));
        }
    }
    for data in salvage_data {
        println!("{}", data)
    }
}
