use yaml_rust2::{Yaml, YamlLoader};

#[derive(Debug)]
pub struct KubeConfig {
    yaml: Vec<Yaml>,
}

pub fn from_str(s: &str) -> KubeConfig {
    let kube_config = YamlLoader::load_from_str(s).unwrap();

    KubeConfig { yaml: kube_config }
}

impl KubeConfig {
    pub fn get_current_context(&self) -> Option<String> {
        let doc = &self.yaml[0];
        if let Some(root) = &doc.as_hash() {
            let key = Yaml::String("current-context".to_string());
            if let Some(context) = root.get(&key) {
                return context.as_str().map(std::string::ToString::to_string);
            }
        }

        None
    }
}
