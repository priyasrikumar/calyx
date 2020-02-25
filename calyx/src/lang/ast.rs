use crate::errors::Error;
use crate::lang::context::LibraryContext;
use sexpy::Sexpy;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Abstract Syntax Tree for Futil. See link below for the grammar
// https://github.com/cucapra/futil/blob/master/grammar.md

pub type Id = String;

pub fn parse_file(file: &PathBuf) -> Result<NamespaceDef, Error> {
    let content = &fs::read(file)?;
    let string_content = std::str::from_utf8(content)?;
    match NamespaceDef::parse(string_content) {
        Ok(ns) => Ok(ns),
        Err(msg) => Err(Error::ParseError(msg)),
    }
}

#[derive(Clone, Debug, Hash, Sexpy)]
#[sexpy(head = "define/namespace")]
pub struct NamespaceDef {
    pub name: String,
    pub components: Vec<ComponentDef>,
}

#[derive(Clone, Debug, Hash, Sexpy)]
#[sexpy(head = "define/component")]
pub struct ComponentDef {
    pub name: Id,
    pub signature: Signature,
    #[sexpy(surround)]
    pub structure: Vec<Structure>,
    pub control: Control,
}

impl ComponentDef {
    /// Given a Library Context, resolve all the primitive components
    /// in `self` and return the signatures in a HashMap
    pub fn resolve_primitives(
        &self,
        libctx: &LibraryContext,
    ) -> Result<HashMap<Id, Signature>, Error> {
        let mut map = HashMap::new();

        for stmt in &self.structure {
            if let Structure::Std { data } = stmt {
                let sig = libctx
                    .resolve(&data.instance.name, &data.instance.params)?;
                map.insert(data.name.clone(), sig);
            }
        }

        Ok(map)
    }
}

#[derive(Clone, Debug, Hash, Sexpy)]
#[sexpy(nohead, nosurround)]
pub struct Signature {
    #[sexpy(surround)]
    pub inputs: Vec<Portdef>,
    #[sexpy(surround)]
    pub outputs: Vec<Portdef>,
}

impl Signature {
    /// Returns an iterator over the inputs of signature
    pub fn inputs(&self) -> std::slice::Iter<Portdef> {
        self.inputs.iter()
    }

    /// Returns an iterator over the outputs of signature
    pub fn outputs(&self) -> std::slice::Iter<Portdef> {
        self.outputs.iter()
    }

    pub fn new(inputs: &[(&str, u64)], outputs: &[(&str, u64)]) -> Self {
        Signature {
            inputs: inputs.iter().map(|x| x.into()).collect(),
            outputs: outputs.iter().map(|x| x.into()).collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, Sexpy, PartialEq)]
#[sexpy(head = "port")]
pub struct Portdef {
    pub name: String,
    pub width: u64,
}

impl From<(String, u64)> for Portdef {
    fn from((name, width): (String, u64)) -> Self {
        Portdef { name, width }
    }
}

impl From<(&str, u64)> for Portdef {
    fn from((name, width): (&str, u64)) -> Self {
        Portdef {
            name: name.to_string(),
            width,
        }
    }
}

impl From<&(&str, u64)> for Portdef {
    fn from((name, width): &(&str, u64)) -> Self {
        Portdef {
            name: name.to_string(),
            width: *width,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Sexpy)]
#[sexpy(head = "@")]
pub enum Port {
    Comp {
        component: Id,
        port: String,
    },
    #[sexpy(head = "this")]
    This {
        port: String,
    },
}

#[derive(Clone, Debug, Hash, Sexpy, PartialEq)]
#[sexpy(nohead)]
pub struct Compinst {
    pub name: Id,
    pub params: Vec<u64>,
}

// ===================================
// Data definitions for Structure
// ===================================

#[derive(Clone, Debug, Hash, Sexpy, PartialEq)]
#[sexpy(head = "new", nosurround)]
pub struct Decl {
    pub name: Id,
    pub component: String,
}

#[derive(Clone, Debug, Hash, Sexpy, PartialEq)]
#[sexpy(head = "new-std", nosurround)]
pub struct Std {
    pub name: Id,
    pub instance: Compinst,
}

#[derive(Clone, Debug, Hash, Sexpy, PartialEq)]
#[sexpy(head = "->", nosurround)]
pub struct Wire {
    pub src: Port,
    pub dest: Port,
}

#[derive(Clone, Debug, Hash, Sexpy, PartialEq)]
#[sexpy(nohead)]
pub enum Structure {
    Decl { data: Decl },
    Std { data: Std },
    Wire { data: Wire },
}

#[allow(unused)]
impl Structure {
    pub fn decl(name: Id, component: String) -> Structure {
        Structure::Decl {
            data: Decl { name, component },
        }
    }

    pub fn std(name: Id, instance: Compinst) -> Structure {
        Structure::Std {
            data: Std { name, instance },
        }
    }

    pub fn wire(src: Port, dest: Port) -> Structure {
        Structure::Wire {
            data: Wire { src, dest },
        }
    }
}

// ===================================
// Data definitions for Control Ast
// ===================================
// Need Boxes for recursive data structure
// Cannot have recursive data structure without
// indirection

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nosurround)]
pub struct Seq {
    pub stmts: Vec<Control>,
}

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nosurround)]
pub struct Par {
    pub stmts: Vec<Control>,
}

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nosurround)]
pub struct If {
    pub port: Port,
    #[sexpy(surround)]
    pub cond: Vec<String>,
    pub tbranch: Box<Control>,
    pub fbranch: Box<Control>,
}

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nosurround)]
pub struct Ifen {
    pub port: Port,
    #[sexpy(surround)]
    pub cond: Vec<String>,
    pub tbranch: Box<Control>,
    pub fbranch: Box<Control>,
}

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nosurround)]
pub struct While {
    pub port: Port,
    #[sexpy(surround)]
    pub cond: Vec<String>,
    pub body: Box<Control>,
}

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nosurround)]
pub struct Print {
    pub var: String,
}

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nosurround)]
pub struct Enable {
    pub comps: Vec<String>,
}

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nosurround)]
pub struct Disable {
    pub comps: Vec<String>,
}

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nosurround)]
pub struct Empty {}

#[derive(Debug, Clone, Hash, Sexpy)]
#[sexpy(nohead)]
pub enum Control {
    Seq { data: Seq },
    Par { data: Par },
    If { data: If },
    Ifen { data: Ifen },
    While { data: While },
    Print { data: Print },
    Enable { data: Enable },
    Disable { data: Disable },
    Empty { data: Empty },
}

#[allow(unused)]
impl Control {
    pub fn seq(stmts: Vec<Control>) -> Control {
        Control::Seq {
            data: Seq { stmts },
        }
    }

    pub fn par(stmts: Vec<Control>) -> Control {
        Control::Par {
            data: Par { stmts },
        }
    }

    // pub fn c_if(cond: Port, tbranch: Control, fbranch: Control) -> Control {
    //     Control::If {
    //         data: If {
    //             cond,
    //             tbranch: Box::new(tbranch),
    //             fbranch: Box::new(fbranch),
    //         },
    //     }
    // }

    // pub fn ifen(cond: Port, tbranch: Control, fbranch: Control) -> Control {
    //     Control::Ifen {
    //         data: Ifen {
    //             cond,
    //             tbranch: Box::new(tbranch),
    //             fbranch: Box::new(fbranch),
    //         },
    //     }
    // }

    // pub fn c_while(cond: Port, body: Control) -> Control {
    //     Control::While {
    //         data: While {
    //             cond,
    //             body: Box::new(body),
    //         },
    //     }
    // }

    pub fn print(var: String) -> Control {
        Control::Print {
            data: Print { var },
        }
    }

    pub fn enable(comps: Vec<String>) -> Control {
        Control::Enable {
            data: Enable { comps },
        }
    }

    pub fn disable(comps: Vec<String>) -> Control {
        Control::Disable {
            data: Disable { comps },
        }
    }

    pub fn empty() -> Control {
        Control::Empty { data: Empty {} }
    }
}
