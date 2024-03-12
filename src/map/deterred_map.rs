//! Macro collection for deterred macros
use crate::argument::{MacroInput, ValueType};
use crate::common::{ETMap, ETable, RadResult};
use crate::consts::ESR;
use crate::extension::{ExtMacroBody, ExtMacroBuilder};
use crate::{man_det, AuthType};
use crate::{Parameter, Processor};
#[cfg(feature = "rustc_hash")]
use rustc_hash::FxHashMap as HashMap;
#[cfg(not(feature = "rustc_hash"))]
use std::collections::HashMap;

use std::iter::FromIterator;

/// Function signature for a deterred macro function
pub(crate) type DFunctionMacroType = fn(MacroInput, &mut Processor) -> RadResult<Option<String>>;

/// Collection map for a deterred macro function
#[derive(Clone)]
pub struct DeterredMacroMap {
    pub(crate) macros: HashMap<String, DMacroSign>,
}

impl DeterredMacroMap {
    /// Creates empty map
    pub fn empty() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    /// Create a new instance with default macros
    pub fn new() -> Self {
        Self::from_iter(IntoIterator::into_iter([
            (DMacroSign::new(
                "include",
                [(ValueType::Path, "a_filename")],
                Self::include,
                Some(man_det!("include.r4d")),
            )
            .optional(Parameter::new(ValueType::Bool, "a_raw_mode"))
            .require_auth(&[AuthType::FIN])),
            (DMacroSign::new(
                "read",
                [(ValueType::Path, "a_filename")],
                Self::incread,
                Some(man_det!("read.r4d")),
            )
            .optional(Parameter::new(ValueType::Bool, "a_raw_mode"))
            .require_auth(&[AuthType::FIN])),
            (DMacroSign::new(
                "bufread",
                [(ValueType::Path, "a_file")],
                DeterredMacroMap::read_in,
                Some(man_det!("bufread.r4d")),
            )
            .optional(Parameter::new(ValueType::Bool, "a_raw_mode"))
            .require_auth(&[AuthType::FIN])),
            (DMacroSign::new(
                "readto",
                [
                    (ValueType::Path, "a_from_file"),
                    (ValueType::Path, "a_to_file"),
                ],
                DeterredMacroMap::read_to,
                Some(man_det!("readto.r4d")),
            )
            .optional(Parameter::new(ValueType::Bool, "a_raw_mode"))
            .no_ret()
            .require_auth(&[AuthType::FIN, AuthType::FOUT])),
            (DMacroSign::new(
                "tempin",
                ESR,
                Self::temp_include,
                Some(man_det!("tempin.r4d")),
            )
            .optional(Parameter::new(ValueType::Bool, "a_raw_mode"))
            .require_auth(&[AuthType::FIN])),
            (DMacroSign::new(
                "mapf",
                [
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Path, "a_file"),
                ],
                Self::map_file,
                Some(man_det!("mapf.r4d")),
            )
            .require_auth(&[AuthType::FIN])),
            (DMacroSign::new(
                "mapfe",
                [
                    (ValueType::Regex, "a_expr"),
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Text, "a_lines"),
                ],
                Self::map_file_expr,
                None,
            )
            .require_auth(&[AuthType::FIN])),
            #[cfg(feature = "evalexpr")]
            (DMacroSign::new(
                "mapn",
                [
                    (ValueType::CText, "a_operation"),
                    (ValueType::Text, "a_source"),
                ],
                Self::map_number,
                None,
            )),
            (DMacroSign::new(
                "ifenv",
                [
                    (ValueType::CText, "a_env_name"),
                    (ValueType::Text, "a_if_expr"),
                ],
                DeterredMacroMap::ifenv,
                Some(man_det!("ifenv.r4d")),
            )
            .require_auth(&[AuthType::ENV])),
            (DMacroSign::new(
                "ifenvel",
                [
                    (ValueType::CText, "a_env_name"),
                    (ValueType::Text, "a_if_expr"),
                    (ValueType::Text, "a_else_expr"),
                ],
                DeterredMacroMap::ifenvel,
                Some(man_det!("ifenvel.r4d")),
            )
            .require_auth(&[AuthType::ENV])),
            (DMacroSign::new(
                "append",
                [
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Text, "a_content"),
                ],
                Self::append,
                Some(man_det!("append.r4d")),
            )
            .no_ret()),
            (DMacroSign::new(
                "anon",
                [(ValueType::Text, "a_macro_definition")],
                Self::add_anonymous_macro,
                Some(man_det!("anon.r4d")),
            )),
            (DMacroSign::new(
                "stream",
                [(ValueType::CText, "a_macro_name")],
                Self::stream,
                Some(man_det!("stream.r4d")),
            )
            .no_ret()),
            (DMacroSign::new("consume", ESR, Self::consume, Some(man_det!("consume.r4d")))),
            (DMacroSign::new(
                "EB",
                ESR,
                DeterredMacroMap::escape_blanks,
                Some(man_det!("EB.r4d")),
            )
            .no_ret()),
            (DMacroSign::new(
                "exec",
                [
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::CText, "a_macro_attribute"),
                    (ValueType::Text, "a_macro_args"),
                ],
                DeterredMacroMap::execute_macro,
                Some(man_det!("exec.r4d")),
            )),
            (DMacroSign::new(
                "fassert",
                [(ValueType::Text, "a_expr")],
                DeterredMacroMap::assert_fail,
                Some(man_det!("fassert.r4d")),
            )
            .no_ret()),
            (DMacroSign::new(
                "forby",
                [
                    (ValueType::Text, "a_body"),
                    (ValueType::Text, "a_sep"),
                    (ValueType::Text, "a_text"),
                ],
                DeterredMacroMap::forby,
                Some(man_det!("forby.r4d")),
            )),
            (DMacroSign::new(
                "foreach",
                [(ValueType::Text, "a_body"), (ValueType::Text, "a_array")],
                DeterredMacroMap::foreach,
                Some(man_det!("foreach.r4d")),
            )),
            (DMacroSign::new(
                "forjoin",
                [
                    (ValueType::Text, "a_body"),
                    (ValueType::Text, "a_joined_array"),
                ],
                DeterredMacroMap::forjoin,
                None,
            )),
            (DMacroSign::new(
                "forsp",
                [(ValueType::Text, "a_body"), (ValueType::Text, "a_words")],
                DeterredMacroMap::for_space,
                Some(man_det!("forsp.r4d")),
            )),
            (DMacroSign::new(
                "forline",
                [(ValueType::Text, "a_body"), (ValueType::Text, "a_lines")],
                DeterredMacroMap::forline,
                Some(man_det!("forline.r4d")),
            )),
            (DMacroSign::new(
                "forcol",
                [(ValueType::Text, "a_body"), (ValueType::Text, "a_table")],
                DeterredMacroMap::forcol,
                None,
            )),
            (DMacroSign::new(
                "forloop",
                [
                    (ValueType::Text, "a_body"),
                    (ValueType::Uint, "a_min"),
                    (ValueType::Uint, "a_max"),
                ],
                DeterredMacroMap::forloop,
                Some(man_det!("forloop.r4d")),
            )),
            (DMacroSign::new(
                "map",
                [
                    (ValueType::Regex, "a_expr"),
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Text, "a_text"),
                ],
                Self::map,
                None,
            )),
            (DMacroSign::new(
                "mape",
                [
                    (ValueType::Regex, "a_start_expr"),
                    (ValueType::Text, "a_end_expr"),
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Text, "a_source"),
                ],
                Self::map_expression,
                None,
            )),
            (DMacroSign::new(
                "mapa",
                [
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Text, "a_array"),
                ],
                Self::map_array,
                Some(man_det!("mapa.r4d")),
            )),
            (DMacroSign::new(
                "mapl",
                [
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Text, "a_lines"),
                ],
                Self::map_lines,
                Some(man_det!("mapl.r4d")),
            )),
            (DMacroSign::new(
                "maple",
                [
                    (ValueType::Regex, "a_expr"),
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Text, "a_lines"),
                ],
                Self::map_lines_expr,
                None,
            )),
            (DMacroSign::new(
                "spread",
                [
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::CText, "a_csv_value"),
                ],
                Self::spread_data,
                Some(man_det!("spread.r4d")),
            )),
            (DMacroSign::new(
                "streaml",
                [(ValueType::CText, "a_macro_name")],
                Self::stream_by_lines,
                Some(man_det!("streaml.r4d")),
            )
            .no_ret()),
            (DMacroSign::new(
                "if",
                [(ValueType::Bool, "a_cond"), (ValueType::Text, "a_if_expr")],
                DeterredMacroMap::if_cond,
                Some(man_det!("if.r4d")),
            )),
            (DMacroSign::new(
                "ifelse",
                [
                    (ValueType::Bool, "a_cond"),
                    (ValueType::Text, "a_if_expr"),
                    (ValueType::Text, "a_else_expr"),
                ],
                DeterredMacroMap::ifelse,
                Some(man_det!("ifelse.r4d")),
            )),
            (DMacroSign::new(
                "ifdef",
                [
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Text, "a_if_expr"),
                ],
                DeterredMacroMap::ifdef,
                Some(man_det!("ifdef.r4d")),
            )),
            (DMacroSign::new(
                "ifdefel",
                [
                    (ValueType::CText, "a_macro_name"),
                    (ValueType::Text, "a_if_expr"),
                    (ValueType::Text, "a_else_expr"),
                ],
                DeterredMacroMap::ifdefel,
                Some(man_det!("ifdefel.r4d")),
            )),
            (DMacroSign::new(
                "logm",
                [(ValueType::CText, "a_macro_name")],
                Self::log_macro_info,
                Some(man_det!("logm.r4d")),
            )
            .no_ret()),
            (DMacroSign::new(
                "que",
                [(ValueType::Text, "a_expr")],
                DeterredMacroMap::queue_content,
                Some(man_det!("que.r4d")),
            )
            .no_ret()),
            (DMacroSign::new(
                "queif",
                [(ValueType::Bool, "a_bool"), (ValueType::Text, "a_content")],
                DeterredMacroMap::if_queue_content,
                Some(man_det!("queif.r4d")),
            )
            .no_ret()),
            (DMacroSign::new(
                "expand",
                [(ValueType::Text, "a_literal_expr")],
                DeterredMacroMap::expand_expression,
                Some(man_det!("expand.r4d")),
            )),
        ]))
    }

    /// Get Function pointer from map
    pub fn get_deterred_macro(&self, name: &str) -> Option<&DFunctionMacroType> {
        if let Some(mac) = self.macros.get(name) {
            Some(&mac.logic)
        } else {
            None
        }
    }

    /// Get Function pointer from map
    pub(crate) fn get_signature(&self, name: &str) -> Option<&DMacroSign> {
        self.macros.get(name)
    }

    /// Check if map contains the name
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Undefine a deterred macro
    pub fn undefine(&mut self, name: &str) {
        self.macros.remove(name);
    }

    /// Rename a deterred macro
    pub fn rename(&mut self, name: &str, target: &str) -> bool {
        if let Some(func) = self.macros.remove(name) {
            self.macros.insert(target.to_owned(), func);
            return true;
        }
        false
    }

    /// Add new extension macro as deterred macro
    pub fn new_ext_macro(&mut self, ext: ExtMacroBuilder) {
        // TODO TT
        // if let Some(ExtMacroBody::Deterred(mac_ref)) = ext.macro_body {
        //     let sign = DMacroSign::new(&ext.macro_name, &ext.args, mac_ref, ext.macro_desc);
        //     self.macros.insert(ext.macro_name, sign);
        // }
    }
}

#[derive(Clone)]
pub(crate) struct DMacroSign {
    name: String,
    params: Vec<Parameter>,
    optional: Option<Parameter>,
    optional_multiple: bool,
    pub enum_table: ETMap,
    pub logic: DFunctionMacroType,
    #[allow(dead_code)]
    desc: Option<String>,
    pub ret: ValueType,
    pub required_auth: Vec<AuthType>,
}

impl DMacroSign {
    pub fn new(
        name: &str,
        params: impl IntoIterator<Item = (ValueType, impl AsRef<str>)>,
        logic: DFunctionMacroType,
        desc: Option<String>,
    ) -> Self {
        let params = params
            .into_iter()
            .map(|(t, s)| Parameter {
                name: s.as_ref().to_string(),
                arg_type: t,
            })
            .collect::<Vec<Parameter>>();
        Self {
            name: name.to_owned(),
            params,
            optional: None,
            optional_multiple: false,
            enum_table: ETMap::default(),
            logic,
            desc,
            ret: ValueType::Text,
            required_auth: vec![],
        }
    }

    pub fn no_ret(mut self) -> Self {
        self.ret = ValueType::None;
        self
    }

    pub fn require_auth(mut self, auths: &[AuthType]) -> Self {
        self.required_auth = auths.to_vec();
        self
    }

    pub fn ret(mut self, ret_type: ValueType) -> Self {
        self.ret = ret_type;
        self
    }

    pub fn enum_table(mut self, table: (String, ETable)) -> Self {
        self.enum_table.tables.insert(table.0, table.1);
        self
    }

    pub fn optional(mut self, param: Parameter) -> Self {
        self.optional.replace(param);
        self
    }

    pub fn optional_multiple(mut self) -> Self {
        self.optional_multiple = true;
        self
    }
}

// ------ REFACTOR

impl From<&DMacroSign> for crate::sigmap::MacroSignature {
    fn from(ms: &DMacroSign) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Deterred,
            name: ms.name.to_owned(),
            params: ms.params.to_owned(),
            optional: ms.optional.clone(),
            optional_multiple: ms.optional_multiple,
            enum_table: ms.enum_table.clone(),
            desc: ms.desc.clone(),
            return_type: ms.ret,
            required_auth: ms.required_auth.clone(),
        }
    }
}

impl FromIterator<DMacroSign> for DeterredMacroMap {
    fn from_iter<T: IntoIterator<Item = DMacroSign>>(iter: T) -> Self {
        let mut m = HashMap::new();
        for sign in iter {
            m.insert(sign.name.clone(), sign);
        }
        Self { macros: m }
    }
}
