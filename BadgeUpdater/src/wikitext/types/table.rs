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
    /// Raw attributes string if present (e.g. `data-sort-value="3"`).
    pub attrs: Option<String>,
}

impl TableCell {
    pub fn new(content: ParsedData) -> Self {
        Self {
            content,
            rowspan: 1,
            colspan: 1,
            attrs: None,
        }
    }
}

/// Lightweight wrapper around a `TableCell` providing convenience accessors.
#[derive(Debug, Clone)]
pub struct Cell {
    pub inner: TableCell,
}

impl Cell {
    pub fn new(inner: TableCell) -> Self {
        Self { inner }
    }

    /// Return the raw attribute string for this cell (for example `data-sort-value="3"`).
    pub fn get_class(&self) -> String {
        self.inner.attrs.clone().unwrap_or_default()
    }

    /// Return the parsed content of the cell as `ParsedData`.
    pub fn get_parsed(&self) -> ParsedData {
        self.inner.content.clone()
    }

    /// Return the raw textual content of the cell (unparsed).
    pub fn raw(&self) -> String {
        self.inner.content.raw.clone()
    }
}

/// Row wrapper that keeps a handle to the parent table and the row index.
#[derive(Debug, Clone)]
pub struct Row {
    pub table: Table,
    pub idx: usize,
}

impl Row {
    pub fn new(table: Table, idx: usize) -> Self {
        Self { table, idx }
    }

    /// Reconstruct a best-effort raw representation for this row by joining cell wikitexts.
    pub fn raw(&self) -> String {
        let row = &self.table.rows[self.idx];
        let mut out = String::new();
        let mut first = true;
        for cell in row {
            if !first {
                out.push_str(" || ");
            }
            out.push_str(&cell.content.to_wikitext());
            first = false;
        }
        out
    }

    /// Get a cell from this row by header name or column index.
    pub fn get_cell_from_col(&self, col: &str) -> Option<Cell> {
        self.table.get_cell(self.idx, col)
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

    /// Return a wrapper `Row` for the given row index if it exists.
    pub fn get_row(&self, row_idx: usize) -> Option<Row> {
        if row_idx < self.rows.len() {
            Some(Row::new(self.clone(), row_idx))
        } else {
            None
        }
    }

    /// Get a cell by row index and column identifier (either numeric index as string
    /// or header name). Returns a cloned `Cell` if present.
    pub fn get_cell(&self, row_idx: usize, col: &str) -> Option<Cell> {
        if let Ok(ci) = col.parse::<usize>() {
            return self.get_cell_by_index(row_idx, ci).map(Cell::new);
        }
        // search headers case-insensitive
        for (i, h) in self.headers.iter().enumerate() {
            if h.eq_ignore_ascii_case(col) {
                return self.get_cell_by_index(row_idx, i).map(Cell::new);
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

/// Find a top-level occurrence of `c` in `s` (not inside nested constructs).
/// Returns the byte index of the top-level occurrence suitable for slicing.
fn find_top_level_char(s: &str, c: char) -> Option<usize> {
    let chs: Vec<(usize, char)> = s.char_indices().collect();
    let mut i = 0usize;
    let n = chs.len();
    let mut depth_brace = 0usize;
    let mut depth_bracket = 0usize;
    let mut in_tag = false;

    while i < n {
        let (byte_pos, ch) = chs[i];
        if ch == '{' && i + 1 < n && chs[i + 1].1 == '{' {
            depth_brace += 1;
            i += 2;
            continue;
        } else if ch == '}' && i + 1 < n && chs[i + 1].1 == '}' {
            if depth_brace > 0 {
                depth_brace -= 1;
            }
            i += 2;
            continue;
        } else if ch == '[' && i + 1 < n && chs[i + 1].1 == '[' {
            depth_bracket += 1;
            i += 2;
            continue;
        } else if ch == ']' && i + 1 < n && chs[i + 1].1 == ']' {
            if depth_bracket > 0 {
                depth_bracket -= 1;
            }
            i += 2;
            continue;
        } else if ch == '<' {
            in_tag = true;
            i += 1;
            continue;
        } else if ch == '>' {
            in_tag = false;
            i += 1;
            continue;
        }

        if ch == c && depth_brace == 0 && depth_bracket == 0 && !in_tag {
            return Some(byte_pos);
        }
        i += 1;
    }
    None
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
        if let Some(pos) = find_top_level_char(tok, '|') {
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
        // capture raw attributes if present (e.g. data-sort-value="3")
        if attrs != content && !attrs.is_empty() {
            cell.attrs = Some(attrs.to_string());
        }
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
        // Number of expected columns based on parsed header row(s). When > 0 we use this
        // to decide whether subsequent '|' lines should append cells to the current
        // row (when filling a row across multiple single-cell lines) or start a new row.
        let mut expected_cols: usize = 0;

        // If first non-empty line contains class=... parse it (robustly handles additional attributes)
        if let Some(first_line_end) = content.find('\n') {
            let first_line = content[..first_line_end].trim();
            if let Some(pos) = first_line.find("class=") {
                let mut rest = first_line[pos + 6..].trim();
                let class_value = if rest.starts_with('"') || rest.starts_with('\'') {
                    let quote = rest.chars().next().unwrap();
                    let rest_inner = &rest[1..];
                    if let Some(end) = rest_inner.find(quote) {
                        rest_inner[..end].to_string()
                    } else if let Some(end2) = rest.find(char::is_whitespace) {
                        rest[..end2]
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string()
                    } else {
                        rest.trim_matches('"').trim_matches('\'').to_string()
                    }
                } else {
                    if let Some(end2) = rest.find(char::is_whitespace) {
                        rest[..end2].to_string()
                    } else {
                        rest.to_string()
                    }
                };
                if !class_value.is_empty() {
                    class = Some(class_value);
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
                // header row: supports both "!!" separators and single-line headers.
                let hdr_line = line.trim_start_matches('!').trim();
                let parts: Vec<&str> = hdr_line.split("!!").collect();
                let mut hrow: Vec<TableCell> = Vec::new();
                for p in parts {
                    let tok = p.trim();
                    let mut attrs = tok;
                    let mut content = tok;
                    if let Some(pos) = find_top_level_char(tok, '|') {
                        attrs = tok[..pos].trim();
                        content = tok[pos + 1..].trim();
                    }
                    let pd = parse_wikitext_fragment(content)
                        .unwrap_or_else(|_| ParsedData::new(content.to_string()));
                    let mut cell = TableCell::new(pd);
                    if attrs != content && !attrs.is_empty() {
                        cell.attrs = Some(attrs.to_string());
                    }
                    // parse colspan/rowspan if provided
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
                    hrow.push(cell);
                }
                // If this header row has a single cell spanning multiple columns, treat it as a table title/caption
                if hrow.len() == 1 && hrow[0].colspan > 1 {
                    title = Some(hrow[0].content.to_wikitext());
                    rows.push(hrow);
                    continue;
                } else {
                    // push header names (visible content) into headers
                    for c in &hrow {
                        headers.push(c.content.to_wikitext());
                    }
                    // Update expected column count now that we know the headers.
                    expected_cols = headers.len();
                    rows.push(hrow);
                    continue;
                }
            }
            if line.starts_with('|') {
                // Parse cells into a temporary vector first so we can decide whether to append
                // to the current row or to start a new one based on expected column count.
                let mut tmp: Vec<TableCell> = Vec::new();
                parse_table_cells_into(line, &mut tmp);
                if tmp.is_empty() {
                    // nothing to add
                    continue;
                }
                if rows.is_empty() {
                    // no row started yet => start one with parsed cells
                    rows.push(tmp);
                    continue;
                }
                // If the last row is empty, append into it.
                if rows.last().map(|r| r.is_empty()).unwrap_or(false) {
                    let current = rows.last_mut().unwrap();
                    current.extend(tmp);
                    continue;
                }
                // Last row already has cells: decide based on expected column count.
                if expected_cols > 0 {
                    let current_len = rows.last().unwrap().len();
                    // if appending fits within expected count, append; otherwise start a new row
                    if current_len + tmp.len() <= expected_cols {
                        let current = rows.last_mut().unwrap();
                        current.extend(tmp);
                    } else {
                        rows.push(tmp);
                    }
                } else {
                    // Unknown expected columns: preserve original behavior (start a new row).
                    rows.push(tmp);
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mini_tower_table() {
        let s = r#"{| class="sortable mw-collapsible mw-collapsed wikitable" width="100%" style="text-align:center;"
! colspan="4" |Mini Tower List
|-
!Difficulty
!Name
!Location
!Difficulty Num
|-
| data-sort-value="3" |{{Difficulty|3}}
|TNF - [[Tower Not Found]]
|{{Emblem|R0}} [[Ring 0]]
|3.11
|-
| data-sort-value="1" |{{Difficulty|1}}
|NEAT - [[Not Even A Tower]]
|{{Emblem|R1}} [[Ring 1]]
|1.11
|-
| data-sort-value="3" |{{Difficulty|3}}
|TIPAT - [[This Is Probably A Tower]]
|{{Emblem|FR}} [[Forgotten Ridge]]
|3.61
|-
| data-sort-value="1" |{{Difficulty|1}}
|MAT - [[Maybe A Tower]]
|{{Emblem|R2}} [[Ring 2]]
|1.07
|-
| data-sort-value="5" |{{Difficulty|5}}
|NEAF - [[Not Even A Flower]]
|{{Emblem|GoE}} [[Garden of Eeshöl]]
|5.79
|}"#;

        let pd = parse_wikitext_fragment(s).expect("parse");
        let tables = pd.get_tables();
        assert_eq!(tables.len(), 1);
        let tb = &tables[0];
        assert_eq!(
            tb.class.as_deref(),
            Some("sortable mw-collapsible mw-collapsed wikitable")
        );
        assert_eq!(tb.title.as_deref(), Some("Mini Tower List"));

        let headers = tb.get_headers();
        assert_eq!(
            headers,
            vec![
                "Difficulty".to_string(),
                "Name".to_string(),
                "Location".to_string(),
                "Difficulty Num".to_string()
            ]
        );

        // find first data row by looking for the Difficulty template in the first column.
        let mut data_row: Option<usize> = None;
        for (i, _r) in tb.get_rows().iter().enumerate() {
            if let Some(c) = tb.get_cell_by_index(i, 0) {
                if c.content.get_template("Difficulty").is_ok() {
                    data_row = Some(i);
                    break;
                }
            }
        }
        let r_idx = data_row.expect("should find a data row with Difficulty template");

        let name_cell = tb.get_cell(r_idx, "Name").expect("name cell should exist");
        assert!(name_cell.raw().contains("TNF - [[Tower Not Found]]"));

        let diff_cell = tb
            .get_cell(r_idx, "Difficulty")
            .expect("difficulty cell should exist");
        assert_eq!(diff_cell.get_class(), "data-sort-value=\"3\"");

        let loc_cell = tb.get_cell(r_idx, "Location").expect("location cell");
        assert!(loc_cell.get_parsed().get_template("Emblem").is_ok());
        let links = loc_cell.get_parsed().get_links(None);
        assert!(links.iter().any(|l| l.label == "Ring 0"));

        let row = tb.get_row(r_idx).expect("row wrapper");
        let row_raw = row.raw();
        assert!(row_raw.contains("{{Difficulty|3}}"));
        assert!(row_raw.contains("TNF - [[Tower Not Found]]"));
        assert!(row_raw.contains("{{Emblem|R0}}"));
        assert!(row_raw.contains("3.11"));
    }
}
