/*
types/table.rs

Table and TableCell types extracted from parsed_data.rs.

Note: This file purposely references `ParsedData` from the sibling `parsed_data`
module (path: crate::wikitext::parsed_data::ParsedData). The module layout in
the parent `wikitext` module must expose this file under `types::table` for the
re-exports in `parsed_data.rs` to work correctly.
*/

use crate::wikitext::parsed_data::{ParsedData, parse_wikitext_fragment};

/// A table cell with potential rowspan/colspan and parsed content.
#[derive(Debug, Clone)]
pub struct TableCell {
    pub content: ParsedData,
    pub rowspan: usize,
    pub colspan: usize,
}

impl TableCell {
    pub fn new(content: ParsedData) -> Self {
        Self {
            content,
            rowspan: 1,
            colspan: 1,
        }
    }
}

/// Table node representing a top-level wikitext table.
#[derive(Debug, Clone)]
pub struct Table {
    pub title: Option<String>,
    pub class: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<TableCell>>,
}

impl Table {
    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }
    pub fn class(&self) -> Option<String> {
        self.class.clone()
    }
    pub fn get_headers(&self) -> Vec<String> {
        self.headers.clone()
    }
    pub fn get_rows(&self) -> Vec<Vec<TableCell>> {
        self.rows.clone()
    }

    /// Return columns as vectors of cells; expands row/col spans so that each
    /// cell position is filled (cells cloned when spanning).
    pub fn get_cols(&self) -> Vec<Vec<TableCell>> {
        let grid = build_table_grid(self);
        if grid.is_empty() {
            return Vec::new();
        }
        let rows = grid.len();
        let cols = grid[0].len();
        let mut out: Vec<Vec<TableCell>> = vec![Vec::new(); cols];
        for c in 0..cols {
            for r in 0..rows {
                if let Some(cell) = &grid[r][c] {
                    out[c].push(cell.clone());
                } else {
                    out[c].push(TableCell::new(ParsedData::new("")));
                }
            }
        }
        out
    }

    /// Get a cell by row index and column identifier (either numeric index as string
    /// or header name). Returns a cloned `TableCell` if present.
    pub fn get_cell(&self, row_idx: usize, col: &str) -> Option<TableCell> {
        if let Ok(ci) = col.parse::<usize>() {
            return self.get_cell_by_index(row_idx, ci);
        }
        // search headers case-insensitive
        for (i, h) in self.headers.iter().enumerate() {
            if h.eq_ignore_ascii_case(col) {
                return self.get_cell_by_index(row_idx, i);
            }
        }
        None
    }

    pub fn get_cell_by_index(&self, row_idx: usize, col_idx: usize) -> Option<TableCell> {
        let grid = build_table_grid(self);
        if grid.is_empty() {
            return None;
        }
        if row_idx < grid.len() && col_idx < grid[0].len() {
            grid[row_idx][col_idx].clone()
        } else {
            None
        }
    }

    /// Reconstruct a best-effort wikitext representation for this table.
    pub fn to_wikitext(&self) -> String {
        let mut out = String::new();
        if let Some(ref cls) = self.class {
            out.push_str(&format!("{{| class=\"{}\"\n", cls));
        } else {
            out.push_str("{|\n");
        }
        if let Some(ref t) = self.title {
            out.push_str(&format!("|+ {}\n", t));
        }
        if !self.headers.is_empty() {
            out.push('!');
            out.push_str(&self.headers.join(" !! "));
            out.push('\n');
        }
        for r in &self.rows {
            out.push_str("|-\n");
            out.push('|');
            let mut first = true;
            for cell in r {
                if !first {
                    out.push_str(" || ");
                }
                out.push_str(&cell.content.to_wikitext());
                first = false;
            }
            out.push('\n');
        }
        out.push_str("|}\n");
        out
    }
}

/// Build a 2D grid of Option<TableCell> for the table expanding rowspan/colspan.
///
/// Cells are cloned to fill spanned positions. The resulting grid has dimensions
/// rows x cols where cols is the maximal occupied column count.
pub fn build_table_grid(table: &Table) -> Vec<Vec<Option<TableCell>>> {
    let rows_count = table.rows.len();
    // estimate max cols by summing colspans per row
    let mut max_cols = 0usize;
    for r in 0..rows_count {
        let mut csum = 0usize;
        for cell in &table.rows[r] {
            csum += cell.colspan.max(1);
        }
        max_cols = max_cols.max(csum);
    }
    if rows_count == 0 || max_cols == 0 {
        return Vec::new();
    }
    let mut grid: Vec<Vec<Option<TableCell>>> = vec![vec![None; max_cols]; rows_count];

    for r in 0..rows_count {
        let mut c = 0usize;
        for cell in &table.rows[r] {
            // find next free column in row r
            while c < max_cols && grid[r][c].is_some() {
                c += 1;
            }
            if c >= max_cols {
                break;
            }
            // place cell at (r,c) and fill spans
            for rr in r..(r + cell.rowspan) {
                for cc in c..(c + cell.colspan) {
                    if rr < rows_count && cc < max_cols {
                        grid[rr][cc] = Some(cell.clone());
                    }
                }
            }
            c += cell.colspan.max(1);
        }
    }

    grid
}

/// Parse the cells from a table data/header line and append to `row`.
///
/// Accepts a line which may start with '|' (data) or be the remainder after '|-'.
pub fn parse_table_cells_into(line: &str, row: &mut Vec<TableCell>) {
    let mut s = line;
    if s.starts_with('|') {
        s = &s[1..];
    }
    // split on "||" primarily
    let parts: Vec<&str> = s.split("||").collect();
    for part in parts {
        let tok = part.trim();
        if tok.is_empty() {
            row.push(TableCell::new(ParsedData::new("")));
            continue;
        }
        // attributes may appear before a single '|' inside the token, e.g. colspan="2" | value
        let mut attrs = tok;
        let mut content = tok;
        if let Some(pos) = tok.find('|') {
            attrs = tok[..pos].trim();
            content = tok[pos + 1..].trim();
        }
        let parsed = if content.is_empty() {
            ParsedData::new("")
        } else {
            parse_wikitext_fragment(content)
                .unwrap_or_else(|_| ParsedData::new(content.to_string()))
        };
        let mut cell = TableCell::new(parsed);
        // parse simple colspan/rowspan patterns like colspan="2" or rowspan=3
        for attr in attrs.split_whitespace() {
            let a = attr.trim();
            if a.starts_with("colspan=") {
                if let Some(eq) = a.find('=') {
                    let v = a[eq + 1..].trim().trim_matches('"').trim_matches('\'');
                    if let Ok(n) = v.parse::<usize>() {
                        cell.colspan = n.max(1);
                    }
                }
            } else if a.starts_with("rowspan=") {
                if let Some(eq) = a.find('=') {
                    let v = a[eq + 1..].trim().trim_matches('"').trim_matches('\'');
                    if let Ok(n) = v.parse::<usize>() {
                        cell.rowspan = n.max(1);
                    }
                }
            }
        }
        row.push(cell);
    }
}

/// Parse a table starting at `start` (expects "{|"), returns consumed bytes and a `Table`.
///
/// Conservative parser: find matching "|}" and parse common constructs:
/// - initial attribute line (e.g. class="wikitable")
/// - caption "|+"
/// - row separator "|-"
/// - header rows starting with "!"
/// - data rows starting with "|"
pub fn parse_table_at(input: &str, start: usize) -> Option<(usize, Table)> {
    let len = input.len();
    if start + 1 >= len {
        return None;
    }
    if !input[start..].starts_with("{|") {
        return None;
    }
    // find the end of table
    if let Some(rel_end) = input[start + 2..].find("|}") {
        let end_idx = start + 2 + rel_end + 2; // include "|}"
        let content = &input[start + 2..start + 2 + rel_end]; // between "{|" and "|}"

        // parse lines
        let mut class: Option<String> = None;
        let mut title: Option<String> = None;
        let mut headers: Vec<String> = Vec::new();
        let mut rows: Vec<Vec<TableCell>> = Vec::new();

        // If first non-empty line contains class=... parse it
        if let Some(first_line_end) = content.find('\n') {
            let first_line = content[..first_line_end].trim();
            if first_line.starts_with("class=") {
                let v = first_line[6..].trim();
                let v = v.trim_matches('"').trim_matches('\'').trim();
                if !v.is_empty() {
                    class = Some(v.to_string());
                }
            }
        }

        for raw_line in content.lines() {
            let line = raw_line.trim_start();
            if line.is_empty() {
                continue;
            }
            if line.starts_with("|+") {
                title = Some(line[2..].trim().to_string());
                continue;
            }
            if line.starts_with("|-") {
                // start a new (empty) data row
                rows.push(Vec::new());
                // if remainder contains cells (e.g. "|- | a || b"), parse them into current row
                let rest = line[2..].trim();
                if !rest.is_empty() && rest.starts_with('|') {
                    if rows.is_empty() {
                        rows.push(Vec::new());
                    }
                    let current = rows.last_mut().unwrap();
                    parse_table_cells_into(rest, current);
                }
                continue;
            }
            if line.starts_with('!') {
                // header row: split on "!!"
                let hdr_line = line.trim_start_matches('!').trim();
                let parts: Vec<&str> = hdr_line.split("!!").collect();
                let mut hrow: Vec<TableCell> = Vec::new();
                for p in parts {
                    let txt = p.trim();
                    let pd = parse_wikitext_fragment(txt)
                        .unwrap_or_else(|_| ParsedData::new(txt.to_string()));
                    headers.push(txt.to_string());
                    hrow.push(TableCell::new(pd));
                }
                rows.push(hrow);
                continue;
            }
            if line.starts_with('|') {
                // data line: if the previous row already contains cells, start a new row.
                if rows.is_empty() {
                    rows.push(Vec::new());
                }
                // If the last row is non-empty, treat this '|' line as starting a NEW row.
                let need_new_row = rows.last().map(|r| !r.is_empty()).unwrap_or(false);
                if need_new_row {
                    rows.push(Vec::new());
                }
                let current = rows.last_mut().unwrap();
                parse_table_cells_into(line, current);
                continue;
            }
            // unknown line - ignore
        }

        let table = Table {
            title,
            class,
            headers,
            rows,
        };
        return Some((end_idx - start, table));
    }

    None
}
