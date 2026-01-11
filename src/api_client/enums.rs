type ApiVersion = String;

#[derive(Debug, Clone)]
pub enum ApiGroup {
    Core(ApiVersion),
    Named(String, ApiVersion),
}

impl std::fmt::Display for ApiGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiGroup::Core(version) => write!(f, "core/{version}"),
            ApiGroup::Named(name, version) => write!(f, "{name}/{version}"),
        }
    }
}

impl ApiGroup {
    pub fn to_kube_group(&self) -> String {
        match self {
            ApiGroup::Core(version) => format!("io.k8s.api.core.{version}"),
            ApiGroup::Named(name, version) => format!("io.k8s.api.{name}.{version}"),
        }
    }
}

impl<'a> From<(&'a str, &'a str)> for ApiGroup {
    fn from(input: (&'a str, &'a str)) -> ApiGroup {
        match input {
            ("" | "CORE", v) => ApiGroup::Core(v.to_string()),
            (n, v) => ApiGroup::Named(n.to_string(), v.to_string()),
        }
    }
}

impl<'a> From<&'a str> for ApiGroup {
    fn from(input: &'a str) -> ApiGroup {
        if input == "v1" {
            ApiGroup::Core(input.to_string())
        } else {
            let parts: Vec<&str> = input.split('/').collect();
            if parts.len() == 2 {
                ApiGroup::Named(parts[0].to_string(), parts[1].to_string())
            } else {
                ApiGroup::Named(input.to_string(), "v1".to_string())
            }
        }
    }
}
