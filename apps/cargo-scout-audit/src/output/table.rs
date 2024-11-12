use crate::{output::raw_report::json_to_string_opt, utils::detectors_info::LintInfo};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    slice::{Iter, IterMut},
    str::FromStr,
    sync::Mutex,
    vec::Vec,
};
use tera::{Context, Tera};
use terminal_color_builder::OutputFormatter;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Color {
    Default,
    Red,
    Green,
    Blue,
    Cyan,
    Yellow,
    Magenta,
}

impl Color {
    pub fn to_json(&self) -> Value {
        Value::String(self.to_string().into())
    }
    pub fn to_string(&self) -> &str {
        match self {
            Color::Default => "default",
            Color::Red => "red",
            Color::Green => "green",
            Color::Blue => "blue",
            Color::Cyan => "cyan",
            Color::Yellow => "yellow",
            Color::Magenta => "magenta",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[allow(unused)]
pub enum Importance {
    Default,
    Info,
    Header,
    Warning,
    Important,
    Error,
}

#[allow(unused)]
impl Importance {
    pub fn to_json(&self) -> Value {
        Value::String(
            match self {
                Importance::Default => "default",
                Importance::Info => "info",
                Importance::Header => "header",
                Importance::Warning => "warning",
                Importance::Important => "important",
                Importance::Error => "error",
            }
            .into(),
        )
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[allow(unused)]
pub enum SemanticColor {
    Default,
    Color(Color),
    Importance(Importance),
}

#[allow(unused)]
impl SemanticColor {
    pub fn to_json(&self) -> (String, Value) {
        match self {
            SemanticColor::Color(c) => ("color".into(), c.to_json()),
            SemanticColor::Importance(i) => ("importance".into(), i.to_json()),
            _ => ("color".into(), Value::Null),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cell {
    pub content: String,
    pub color: SemanticColor,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Row {
    cells: Vec<Cell>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    header: Row,
    rows: Vec<Row>,
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl Cell {
    pub fn new() -> Cell {
        Cell {
            content: String::new(),
            color: SemanticColor::Default,
        }
    }
    pub fn nonstandard_from_str(s: &str) -> Cell {
        Cell {
            content: String::from_str(s).unwrap(),
            color: SemanticColor::Default,
        }
    }
    pub fn from_string(s: String) -> Cell {
        Cell {
            content: s,
            color: SemanticColor::Default,
        }
    }
    pub fn pad(&self, to_width: usize) -> String {
        let mut ret = self.content.clone();
        let n = self.content.len();
        while ret.len() < to_width {
            ret.push(' ');
        }
        ret
    }
    pub fn to_json(&self, width: usize) -> Value {
        let mut ret = serde_json::Map::<String, Value>::new();
        ret.insert("content".into(), Value::String(self.content.clone()));
        let (name, value) = self.color.to_json();
        ret.insert(name, value);
        ret.insert("w".into(), Value::Number(width.into()));
        Value::Object(ret)
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl Row {
    pub fn new() -> Row {
        Row {
            cells: Vec::<Cell>::new(),
        }
    }
    pub fn from_strs(xs: &[&str]) -> Row {
        Row {
            cells: xs.iter().map(|x| Cell::nonstandard_from_str(x)).collect(),
        }
    }
    pub fn from_strings(xs: &[String]) -> Row {
        Row {
            cells: xs.iter().map(|x| Cell::from_string(x.clone())).collect(),
        }
    }
    pub fn len(&self) -> usize {
        self.cells.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
    pub fn iter(&self) -> Iter<'_, Cell> {
        self.cells.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, Cell> {
        self.cells.iter_mut()
    }
    pub fn get(&self, i: usize) -> &'_ Cell {
        &self.cells[i]
    }
    pub fn get_mut(&mut self, i: usize) -> &'_ mut Cell {
        &mut self.cells[i]
    }
    pub fn set_color(&mut self, color: SemanticColor) {
        for cell in self.iter_mut() {
            cell.color = color;
        }
    }
    pub fn to_json_array(&self, widths: &[usize]) -> Value {
        Value::Array(self.to_json_array2(widths))
    }
    pub fn to_json_array2(&self, widths: &[usize]) -> Vec<Value> {
        let mut ret = Vec::<Value>::new();
        for (i, cell) in self.cells.iter().enumerate() {
            ret.push(cell.to_json(widths[i]));
        }
        ret
    }
    pub fn to_json_object(&self, widths: &[usize]) -> Value {
        let mut ret = serde_json::Map::<String, Value>::new();
        for (i, cell) in self.cells.iter().enumerate() {
            ret.insert(cell.content.clone(), cell.to_json(widths[i]));
        }
        Value::Object(ret)
    }
}

#[allow(unused)]
impl Table {
    pub fn new(header: Row) -> Table {
        Table {
            header,
            rows: Vec::<Row>::new(),
        }
    }
    pub fn add_row(&mut self, row: Row) {
        if row.len() != self.header.len() {
            return;
        }
        self.rows.push(row);
    }
    fn get_column_widths(&self) -> Vec<usize> {
        let n = self.header.len();
        let mut ret = Vec::<usize>::new();
        for i in 0..n {
            let mut max = self.header.get(i).content.len();
            for row in self.rows.iter() {
                max = max.max(row.get(i).content.len());
            }
            ret.push(max);
        }
        ret
    }
    fn print_line(widths: &[usize]) {
        for w in widths.iter() {
            print!("+-");
            print!("{}", "-".repeat(*w));
            print!("-");
        }
        println!("+");
    }
    fn print_row(row: &Row, widths: &[usize]) {
        for (index, cell) in row.iter().enumerate() {
            print!("| ");
            let string = {
                let mut formatter = OutputFormatter::new();
                if let SemanticColor::Color(c) = cell.color {
                    formatter = match c {
                        Color::Default => formatter,
                        Color::Red => formatter.fg().red(),
                        Color::Green => formatter.fg().green(),
                        Color::Blue => formatter.fg().blue(),
                        Color::Cyan => formatter.fg().cyan(),
                        Color::Yellow => formatter.fg().yellow(),
                        Color::Magenta => formatter.fg().magenta(),
                    };
                }
                formatter.text_str(&cell.pad(widths[index])).print()
            };
            print!("{}", string);
            print!(" ");
        }
        println!("|");
    }
    pub fn print(&self) {
        let widths = self.get_column_widths();
        Self::print_line(&widths);
        Self::print_row(&self.header, &widths);
        Self::print_line(&widths);
        for row in self.rows.iter() {
            Self::print_row(row, &widths);
        }
        Self::print_line(&widths);
    }
    fn array_to_object(&self, row: Vec<Value>) -> Value {
        let mut ret = serde_json::Map::<String, Value>::new();
        for (i, cell) in row.iter().enumerate() {
            ret.insert(self.header.get(i).content.clone(), cell.clone());
        }
        Value::Object(ret)
    }
    pub fn to_json_map(&self) -> serde_json::Map<String, Value> {
        let widths = self.get_column_widths();

        let mut ret = serde_json::Map::<String, Value>::new();
        ret.insert(
            "widths".into(),
            Value::Array(widths.iter().map(|x| Value::Number((*x).into())).collect()),
        );

        ret.insert(
            "header_order".into(),
            self.header.iter().map(|x| x.content.clone()).collect(),
        );
        ret.insert("header".into(), self.header.to_json_object(&widths));

        ret.insert("rows".into(), {
            let all_rows: Vec<_> = self
                .rows
                .iter()
                .map(|row| self.array_to_object(row.to_json_array2(&widths)))
                .collect();
            Value::Array(all_rows)
        });

        ret
    }
    pub fn to_json_table(&self) -> Value {
        Value::Object(self.to_json_map())
    }
    pub fn get(&self, i: usize) -> &'_ Row {
        &self.rows[i]
    }
    pub fn get_mut(&mut self, i: usize) -> &'_ mut Row {
        &mut self.rows[i]
    }
}

static mut COLOR_MAP: once_cell::sync::Lazy<Mutex<HashMap<String, Color>>> =
    once_cell::sync::Lazy::new(|| {
        let mut ret = HashMap::<String, Color>::new();
        ret.insert("info".into(), Color::Blue);
        ret.insert("header".into(), Color::Green);
        ret.insert("warning".into(), Color::Yellow);
        ret.insert("important".into(), Color::Red);
        ret.insert("error".into(), Color::Red);
        Mutex::new(ret)
    });

fn map_importance(importance: &String) -> Color {
    unsafe {
        if let Some(c) = COLOR_MAP.lock().unwrap().get(importance) {
            *c
        } else {
            Color::Default
        }
    }
}

fn get_color(o: &serde_json::Map<String, Value>) -> Color {
    if let Some(Value::String(color)) = o.get("color") {
        match color.as_str() {
            "red" => Color::Red,
            "green" => Color::Green,
            "blue" => Color::Blue,
            "cyan" => Color::Cyan,
            "yellow" => Color::Yellow,
            "magenta" => Color::Magenta,
            _ => Color::Default,
        }
    } else if let Some(Value::String(importance)) = o.get("importance") {
        map_importance(importance)
    } else {
        Color::Default
    }
}

fn json_to_string(s: &Value) -> String {
    if let Value::String(s) = s {
        s.clone()
    } else {
        s.to_string().trim_matches('"').to_string()
    }
}

fn json_to_int(n: &Value) -> u64 {
    if let Value::Number(n) = n {
        n.as_u64().unwrap_or(0)
    } else {
        0
    }
}

fn process_object_console(o: &serde_json::Map<String, Value>, pad: bool) -> String {
    let content = o.get("content");
    if content.is_none() {
        return "".into();
    }
    let content = content.unwrap();
    let mut formatter = OutputFormatter::new();
    formatter = match get_color(o) {
        Color::Default => formatter,
        Color::Red => formatter.fg().red(),
        Color::Green => formatter.fg().green(),
        Color::Blue => formatter.fg().blue(),
        Color::Cyan => formatter.fg().cyan(),
        Color::Yellow => formatter.fg().yellow(),
        Color::Magenta => formatter.fg().magenta(),
    };
    let mut padded = json_to_string(content);
    if pad {
        if let Some(w) = o.get("w") {
            let w: usize = usize::try_from(json_to_int(w)).ok().unwrap_or(0);
            while padded.len() < w {
                padded.push(' ');
            }
        }
    }
    formatter.text_str(&padded).print()
}

fn process_object_md(o: &serde_json::Map<String, Value>, pad: bool) -> String {
    let content = o.get("content");
    if content.is_none() {
        return "".into();
    }
    let mut padded = json_to_string(content.unwrap());
    if pad {
        if let Some(w) = o.get("w") {
            let w: usize = usize::try_from(json_to_int(w)).ok().unwrap_or(0);
            while padded.len() < w {
                padded.push(' ');
            }
        }
    }
    let color = get_color(o);
    padded = match color {
        Color::Default => padded,
        _ => format!(
            "<span style=\"color:{}\">{}</span>",
            color.to_string(),
            padded
        ),
    };

    padded
}

fn process_object_html(o: &serde_json::Map<String, Value>, pad: bool) -> String {
    process_object_md(o, pad)
}

fn filter_cell_wrapper<F: FnOnce(&serde_json::Map<String, Value>) -> String>(
    values: &HashMap<String, Value>,
    cb: F,
) -> Result<Value, tera::Error> {
    let cell = values.get("cell");
    if cell.is_none() {
        return Ok(Value::String("".into()));
    }
    let cell = cell.unwrap();
    let s: String = match cell {
        Value::Null => "".into(),
        Value::Bool(x) => format!("{}", x),
        Value::Number(x) => format!("{}", x),
        Value::String(x) => x.into(),
        Value::Array(_) => "".into(),
        Value::Object(o) => cb(o),
    };
    Ok(Value::String(s))
}

fn filter_cell_console(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    filter_cell_wrapper(values, |o| process_object_console(o, false))
}

fn filter_cell_with_padding_console(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    filter_cell_wrapper(values, |o| process_object_console(o, true))
}

fn filter_cell_md(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    filter_cell_wrapper(values, |o| process_object_md(o, false))
}

fn filter_cell_with_padding_md(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    filter_cell_wrapper(values, |o| process_object_md(o, true))
}

fn filter_cell_html(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    filter_cell_wrapper(values, |o| process_object_html(o, false))
}

fn filter_cell_with_padding_html(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    filter_cell_wrapper(values, |o| process_object_html(o, true))
}

fn set_color_maps(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    for (k, v) in values {
        if let Value::String(v) = v {
            let c = match v.as_str() {
                "red" => Some(Color::Red),
                "green" => Some(Color::Green),
                "blue" => Some(Color::Blue),
                "cyan" => Some(Color::Cyan),
                "yellow" => Some(Color::Yellow),
                "magenta" => Some(Color::Magenta),
                _ => None,
            };
            if let Some(c) = c {
                unsafe {
                    COLOR_MAP.lock().unwrap().insert(k.clone(), c);
                }
            }
        }
    }
    Ok(Value::Null)
}

fn header_to_orders(header: &Value) -> HashMap<String, usize> {
    let mut ret = HashMap::<String, usize>::new();
    match header {
        Value::Array(a) => {
            for (i, cell) in a.iter().enumerate() {
                if let Value::String(content) = cell {
                    ret.insert(content.clone(), i);
                }
            }
            ret
        }
        _ => ret,
    }
}

fn default_header_order(header: &Value) -> Vec<String> {
    let mut ret = Vec::<String>::new();
    match header {
        Value::Array(a) => {
            for cell in a.iter() {
                if let Value::String(content) = cell {
                    ret.push(content.clone());
                }
            }
            ret
        }
        _ => ret,
    }
}

fn process_order(values: &HashMap<String, Value>, header: &Value) -> Result<Vec<usize>, ()> {
    let header_order = header_to_orders(header);
    let order = if let Some(Value::Array(o)) = values.get("order") {
        o.iter().map(json_to_string).collect()
    } else {
        default_header_order(header)
    };
    let mut ret = Vec::<usize>::new();
    for o in order.iter() {
        let index = header_order.get(o);
        if index.is_none() {
            return Err(());
        }
        ret.push(*index.unwrap());
    }
    Ok(ret)
}

fn print_separator(values: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    let empty = || Ok(Value::String("".into()));

    let summary = values.get("summary");
    if summary.is_none() {
        return empty();
    }
    let summary = summary.unwrap();

    let header = summary.get("header_order");
    if header.is_none() {
        return empty();
    }
    let header = header.unwrap();

    let widths = summary.get("widths");
    if widths.is_none() {
        return empty();
    }
    let widths = widths.unwrap();
    if !widths.is_array() {
        return empty();
    }
    let widths = match widths {
        Value::Array(x) => x,
        _ => return empty(),
    };

    let order = process_order(values, header);
    if order.is_err() {
        return empty();
    }
    let order = order.unwrap();

    let mut ret = String::new();
    ret.push('+');
    for i in order.iter() {
        let w = widths.get(*i);
        if w.is_none() {
            return empty();
        }
        for _ in 0..(json_to_int(w.unwrap()) + 2) {
            ret.push('-');
        }
        ret.push('+');
    }
    Ok(Value::String(ret))
}

fn register_functions_for_tera_base(tera: &mut Tera) {
    tera.register_function("print_separator", print_separator);
    tera.register_function("set_color_maps", set_color_maps);
}

pub(crate) fn register_functions_for_tera_console(tera: &mut Tera) {
    register_functions_for_tera_base(tera);
    tera.register_function("filter_cell", filter_cell_console);
    tera.register_function("filter_cell_with_padding", filter_cell_with_padding_console);
}

pub(crate) fn register_functions_for_tera_md(tera: &mut Tera) {
    register_functions_for_tera_base(tera);
    tera.register_function("filter_cell", filter_cell_md);
    tera.register_function("filter_cell_with_padding", filter_cell_with_padding_md);
}

pub(crate) fn register_functions_for_tera_html(tera: &mut Tera) {
    register_functions_for_tera_base(tera);
    tera.register_function("filter_cell", filter_cell_html);
    tera.register_function("filter_cell_with_padding", filter_cell_with_padding_html);
}

pub(crate) fn prepare_tera_for_table_render_console(
    tera: &mut Tera,
    context: &mut Context,
    table: &Value,
    name: &str,
) {
    register_functions_for_tera_console(tera);
    context.insert(name, table);
}

pub(crate) fn prepare_tera_for_table_render_html(
    tera: &mut Tera,
    context: &mut Context,
    table: &Value,
    name: &str,
) {
    register_functions_for_tera_html(tera);
    context.insert(name, table);
}

fn count_findings(
    findings: &[Value],
    crate_to_find: &String,
    detectors_info: &HashSet<LintInfo>,
) -> [usize; 4] {
    let mut ret = [0_usize; 4];

    let mut ignored = 0;
    for finding in findings.iter() {
        let krate = json_to_string_opt(finding.get("crate"));
        if krate.is_none() || krate.unwrap() != *crate_to_find {
            continue;
        }
        let code = json_to_string_opt(finding.get("code").and_then(|x| x.get("code")));
        if code.is_none() {
            continue;
        }
        let code = code.unwrap();
        let detector = detectors_info.iter().find(|d| d.id == code);
        if detector.is_none() {
            continue;
        }
        let detector = detector.unwrap();
        *match detector.severity.as_str() {
            "Critical" => &mut ret[0],
            "Medium" => &mut ret[1],
            "Minor" => &mut ret[2],
            "Enhancement" => &mut ret[3],
            _ => &mut ignored,
        } += 1;
    }

    ret
}

pub(crate) fn construct_table(
    findings: &[Value],
    crates: &HashMap<String, bool>,
    detectors_info: &HashSet<LintInfo>,
) -> Table {
    let mut header = Row::from_strs(&[
        "Crate",
        "Status",
        "Critical",
        "Medium",
        "Minor",
        "Enhancement",
    ]);
    header.set_color(SemanticColor::Importance(Importance::Header));
    let mut ret = Table::new(header);

    let crate_order: Vec<String> = crates.iter().map(|(x, _)| x.clone()).sorted().collect();
    for krate in crate_order.iter() {
        let [crit, med, min, enhan] = count_findings(findings, krate, detectors_info);
        let success = *crates.get(krate).unwrap_or(&false);
        let success_string = if success {
            "Analyzed"
        } else {
            "Compilation errors"
        }
        .to_string();

        let row = if !success {
            let mut row = Row::from_strings(&[
                krate.clone(),
                success_string,
                "N/A".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
            ]);
            row.get_mut(1).color = SemanticColor::Importance(Importance::Error);
            row
        } else {
            Row::from_strings(&[
                krate.clone(),
                success_string,
                format!("{crit}"),
                format!("{med}"),
                format!("{min}"),
                format!("{enhan}"),
            ])
        };
        ret.add_row(row);
    }

    ret
}
