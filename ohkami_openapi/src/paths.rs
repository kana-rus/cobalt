use super::{Parameter, RequestBody, Responses, security::SecurityScheme};
use super::_util::{is_false, Map};
use serde::Serialize;

#[derive(Serialize)]
pub struct Paths(
    Map<String, Operations>
);

#[derive(Serialize)]
pub struct Operations(
    Map<&'static str, Operation>
);

#[derive(Serialize, Clone)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    operationId: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    summary:      Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description:  Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    externalDocs: Option<ExternalDoc>,
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    parameters: Vec<Parameter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    requestBody: Option<RequestBody>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    security: Vec<Map<SecuritySchemeName, Vec<&'static str>>>,

    responses: Responses,
}
#[derive(Clone)]
struct SecuritySchemeName(SecurityScheme);
impl PartialEq for SecuritySchemeName {
    fn eq(&self, other: &Self) -> bool {
        self.0.__name__ == other.0.__name__
    }
}
impl Serialize for SecuritySchemeName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0.__name__)
    }
}

#[derive(Serialize, Clone)]
pub struct ExternalDoc {
    pub url: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'static str>
}

impl Paths {
    pub fn new() -> Self {
        Self(Map::new())
    }

    pub fn at(mut self, path: impl Into<String>, operations: Operations) -> Self {
        self.0.insert(path.into(), operations);
        self
    }
}

impl Operations {
    pub fn new() -> Self {
        Self(Map::new())
    }

    pub fn get(mut self, operation: Operation) -> Self {
        self.register("get", operation);
        self
    }
    pub fn put(mut self, operation: Operation) -> Self {
        self.register("put", operation);
        self
    }
    pub fn post(mut self, operation: Operation) -> Self {
        self.register("post", operation);
        self
    }
    pub fn patch(mut self, operation: Operation) -> Self {
        self.register("patch", operation);
        self
    }
    pub fn delete(mut self, operation: Operation) -> Self {
        self.register("delete", operation);
        self
    }
    pub fn options(mut self, operation: Operation) -> Self {
        self.register("options", operation);
        self
    }

    #[doc(hidden)]
    pub fn register(&mut self, method: &'static str, operation: Operation) {
        if matches!(method, "get" | "put" | "post" | "patch" | "delete" | "options") {
            self.0.insert(method, operation);
        }
    }
}

impl Operation {
    pub fn with(responses: Responses) -> Self {
        Self {
            responses,
            operationId:  None,
            tags:         Vec::new(),
            summary:      None,
            description:  None,
            externalDocs: None,
            deprecated:   false,
            parameters:   Vec::new(),
            requestBody:  None,
            security:     Vec::new(),
        }
    }

    pub fn param(mut self, param: Parameter) -> Self {
        self.parameters.push(param);
        self
    }
    #[doc(hidden)]
    pub fn replace_empty_param_name_with(&mut self, name: &'static str) {
        if let Some(empty_param) = self.parameters.iter_mut().find(|p| p.name.is_empty()) {
            empty_param.name = name;
        }
    }

    pub fn requestBody(mut self, requestBody: RequestBody) -> Self {
        self.requestBody = Some(requestBody);
        self
    }

    pub fn security<const N: usize>(mut self, securityScheme: SecurityScheme, scopes: [&'static str; N]) -> Self {
        self.security.push(Map::from_iter([(SecuritySchemeName(securityScheme), scopes.into())]));
        self
    }

    pub fn operationId(mut self, operationId: &'static str) -> Self {
        self.operationId = Some(operationId);
        self
    }
    pub fn tags<const N: usize>(mut self, tags: [&'static str; N]) -> Self {
        self.tags = tags.into();
        self
    }
    pub fn summary(mut self, summary: &'static str) -> Self {
        self.summary = Some(summary);
        self
    }
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }
    pub fn externalDocs(mut self, externalDocs: ExternalDoc) -> Self {
        self.externalDocs = Some(externalDocs);
        self
    }
    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }

    pub fn input(mut self, input: Option<super::Input>) -> Self {
        match input {
            None => self,
            Some(super::Input::Body(body)) => self.requestBody(body),
            Some(super::Input::Param(param)) => self.param(param),
            Some(super::Input::Params(params)) => {
                for param in params {self = self.param(param)}
                self
            }
        }
    }
}
