//! Degree Centrality Plugin - Rust implementation for PluMA
//!
//! Computes degree centrality for each node in a weighted adjacency matrix.
//! Degree centrality is the sum of all edge weights connected to a node.
//!
//! Based on the C++ implementation by movingpictures83/Degree

use pluma_plugin_trait::PluMAPlugin;
use std::ffi::CStr;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::raw::c_char;
use std::path::Path;

/// Degree centrality plugin
pub struct DegreePlugin {
    graph: Vec<f32>,
    nodes: Vec<String>,
    size: usize,
    centrality: Vec<f32>,
}

impl Default for DegreePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DegreePlugin {
    pub fn new() -> Self {
        DegreePlugin {
            graph: Vec::new(),
            nodes: Vec::new(),
            size: 0,
            centrality: Vec::new(),
        }
    }

    /// Parse CSV input file containing weighted adjacency matrix
    ///
    /// Format: First row is header with node names, subsequent rows have
    /// node name in first column followed by edge weights
    pub fn input<P: AsRef<Path>>(&mut self, input_file: P) -> Result<(), String> {
        let file = File::open(input_file.as_ref())
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        let mut reader = BufReader::new(file);

        // Count lines to determine size (skip header)
        let mut line_buf = String::new();
        let mut line_count = 0;

        // Skip header
        reader
            .read_line(&mut line_buf)
            .map_err(|e| format!("Failed to read header: {}", e))?;
        line_buf.clear();

        // Count data lines
        while reader.read_line(&mut line_buf).map_err(|e| format!("Read error: {}", e))? > 0 {
            if !line_buf.trim().is_empty() {
                line_count += 1;
            }
            line_buf.clear();
        }

        if line_count == 0 {
            return Err("No data rows in input file".to_string());
        }

        self.size = line_count;
        self.graph = vec![0.0; self.size * self.size];
        self.nodes = Vec::with_capacity(self.size);

        // Re-read file for parsing
        let file = File::open(input_file.as_ref())
            .map_err(|e| format!("Failed to reopen input file: {}", e))?;
        let mut reader = BufReader::new(file);

        // Skip header again
        line_buf.clear();
        reader.read_line(&mut line_buf).ok();
        line_buf.clear();

        let mut row_idx = 0;
        while reader.read_line(&mut line_buf).map_err(|e| format!("Read error: {}", e))? > 0 {
            if line_buf.trim().is_empty() {
                line_buf.clear();
                continue;
            }

            self.parse_row_fast(&line_buf, row_idx);
            row_idx += 1;
            line_buf.clear();
        }

        Ok(())
    }

    /// Fast row parsing without allocations
    #[inline]
    fn parse_row_fast(&mut self, line: &str, row_idx: usize) {
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut pos = 0;
        let mut col_idx = 0;
        let mut in_quotes = false;
        let mut field_start = 0;

        while pos < len {
            let b = bytes[pos];

            if b == b'"' {
                in_quotes = !in_quotes;
                if in_quotes {
                    field_start = pos + 1;
                }
            } else if b == b',' && !in_quotes {
                if col_idx == 0 {
                    // First column is node name
                    let name = extract_field(bytes, field_start, pos);
                    self.nodes.push(name);
                } else {
                    // Subsequent columns are weights
                    let weight_col = col_idx - 1;
                    if weight_col < self.size {
                        let weight = parse_float_fast(bytes, field_start, pos);
                        let idx = row_idx * self.size + weight_col;
                        self.graph[idx] = weight;
                        // Symmetric
                        let sym_idx = weight_col * self.size + row_idx;
                        self.graph[sym_idx] = weight;
                    }
                }
                col_idx += 1;
                field_start = pos + 1;
            }
            pos += 1;
        }

        // Handle last field
        if col_idx > 0 {
            let weight_col = col_idx - 1;
            if weight_col < self.size {
                let end = if bytes.last() == Some(&b'\n') || bytes.last() == Some(&b'\r') {
                    len - 1
                } else {
                    len
                };
                let weight = parse_float_fast(bytes, field_start, end);
                let idx = row_idx * self.size + weight_col;
                self.graph[idx] = weight;
                let sym_idx = weight_col * self.size + row_idx;
                self.graph[sym_idx] = weight;
            }
        }
    }

    /// Compute degree centrality for each node
    ///
    /// Degree centrality is the sum of all edge weights connected to a node
    #[inline]
    pub fn run(&mut self) {
        self.centrality = vec![0.0; self.size];
        let graph = &self.graph;
        let size = self.size;

        for i in 0..size {
            let row_offset = i * size;
            let mut degree: f32 = 0.0;

            // Process in chunks for better cache utilization
            let row = &graph[row_offset..row_offset + size];
            for &val in row {
                degree += val;
            }

            self.centrality[i] = degree;
        }
    }

    /// Write results sorted by degree centrality
    pub fn output<P: AsRef<Path>>(&self, output_file: P) -> Result<(), String> {
        // Create sorted indices by absolute centrality (descending)
        let mut indices: Vec<usize> = (0..self.size).collect();
        indices.sort_unstable_by(|&a, &b| {
            self.centrality[b]
                .abs()
                .partial_cmp(&self.centrality[a].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let file = File::create(output_file.as_ref())
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "Name\tCentrality\tRank")
            .map_err(|e| format!("Failed to write header: {}", e))?;

        for (rank, &idx) in indices.iter().enumerate() {
            writeln!(
                writer,
                "{}\t{:.6}\t{}",
                self.nodes[idx],
                self.centrality[idx],
                rank + 1
            )
            .map_err(|e| format!("Failed to write row: {}", e))?;
        }

        Ok(())
    }

    /// Get the computed centrality values
    pub fn get_centrality(&self) -> &[f32] {
        &self.centrality
    }

    /// Get node names
    pub fn get_nodes(&self) -> &[String] {
        &self.nodes
    }

    /// Get graph size
    pub fn size(&self) -> usize {
        self.size
    }
}

/// Extract field as string, removing quotes
#[inline]
fn extract_field(bytes: &[u8], start: usize, end: usize) -> String {
    let mut s = start;
    let mut e = end;

    // Skip leading quote
    if s < e && bytes[s] == b'"' {
        s += 1;
    }
    // Skip trailing quote
    if e > s && bytes[e - 1] == b'"' {
        e -= 1;
    }

    String::from_utf8_lossy(&bytes[s..e]).into_owned()
}

/// Fast float parsing from bytes
#[inline]
fn parse_float_fast(bytes: &[u8], start: usize, end: usize) -> f32 {
    let mut s = start;
    let mut e = end;

    // Skip quotes and whitespace
    while s < e && (bytes[s] == b'"' || bytes[s] == b' ' || bytes[s] == b'\t') {
        s += 1;
    }
    while e > s
        && (bytes[e - 1] == b'"'
            || bytes[e - 1] == b' '
            || bytes[e - 1] == b'\t'
            || bytes[e - 1] == b'\r'
            || bytes[e - 1] == b'\n')
    {
        e -= 1;
    }

    if s >= e {
        return 0.0;
    }

    // Fast path for simple floats
    let slice = &bytes[s..e];
    if let Ok(s) = std::str::from_utf8(slice) {
        s.parse().unwrap_or(0.0)
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// PluMA plugin contract (pluma-plugin-trait + dlsym-resolved FFI shims)
// ---------------------------------------------------------------------------

impl PluMAPlugin for DegreePlugin {
    fn input(&mut self, filepath: String) -> Result<(), Box<dyn std::error::Error>> {
        DegreePlugin::input(self, &filepath).map_err(|e| e.into())
    }
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        DegreePlugin::run(self);
        Ok(())
    }
    fn output(&mut self, filepath: String) -> Result<(), Box<dyn std::error::Error>> {
        DegreePlugin::output(self, &filepath).map_err(|e| e.into())
    }
}

#[no_mangle]
pub extern "C" fn Degree_plugin_create() -> *mut std::ffi::c_void {
    Box::into_raw(Box::new(DegreePlugin::new())) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn Degree_plugin_destroy(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr as *mut DegreePlugin);
        }
    }
}

#[no_mangle]
pub extern "C" fn Degree_plugin_input(ptr: *mut std::ffi::c_void, filename: *const c_char) {
    if ptr.is_null() || filename.is_null() {
        return;
    }
    unsafe {
        let plugin = &mut *(ptr as *mut DegreePlugin);
        let s = CStr::from_ptr(filename).to_str().unwrap_or("").to_string();
        if let Err(e) = <DegreePlugin as PluMAPlugin>::input(plugin, s) {
            eprintln!("[Degree] input error: {e}");
        }
    }
}

#[no_mangle]
pub extern "C" fn Degree_plugin_run(ptr: *mut std::ffi::c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let plugin = &mut *(ptr as *mut DegreePlugin);
        if let Err(e) = <DegreePlugin as PluMAPlugin>::run(plugin) {
            eprintln!("[Degree] run error: {e}");
        }
    }
}

#[no_mangle]
pub extern "C" fn Degree_plugin_output(ptr: *mut std::ffi::c_void, filename: *const c_char) {
    if ptr.is_null() || filename.is_null() {
        return;
    }
    unsafe {
        let plugin = &mut *(ptr as *mut DegreePlugin);
        let s = CStr::from_ptr(filename).to_str().unwrap_or("").to_string();
        if let Err(e) = <DegreePlugin as PluMAPlugin>::output(plugin, s) {
            eprintln!("[Degree] output error: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "\"\",\"A\",\"B\",\"C\"").unwrap();
        writeln!(file, "\"A\",1,0.8,0.5").unwrap();
        writeln!(file, "\"B\",0.8,1,0.3").unwrap();
        writeln!(file, "\"C\",0.5,0.3,1").unwrap();
        file
    }

    #[test]
    fn test_new() {
        let plugin = DegreePlugin::new();
        assert_eq!(plugin.size, 0);
        assert!(plugin.graph.is_empty());
    }

    #[test]
    fn test_input() {
        let file = create_test_csv();
        let mut plugin = DegreePlugin::new();
        plugin.input(file.path()).unwrap();

        assert_eq!(plugin.size, 3);
        assert_eq!(plugin.nodes, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_run() {
        let file = create_test_csv();
        let mut plugin = DegreePlugin::new();
        plugin.input(file.path()).unwrap();
        plugin.run();

        // A: 1 + 0.8 + 0.5 = 2.3
        // B: 0.8 + 1 + 0.3 = 2.1
        // C: 0.5 + 0.3 + 1 = 1.8
        let centrality = plugin.get_centrality();
        assert!((centrality[0] - 2.3).abs() < 0.01);
        assert!((centrality[1] - 2.1).abs() < 0.01);
        assert!((centrality[2] - 1.8).abs() < 0.01);
    }

    #[test]
    fn test_output() {
        let file = create_test_csv();
        let mut plugin = DegreePlugin::new();
        plugin.input(file.path()).unwrap();
        plugin.run();

        let output_file = NamedTempFile::new().unwrap();
        plugin.output(output_file.path()).unwrap();

        let contents = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(contents.contains("Name\tCentrality\tRank"));
        assert!(contents.contains("A"));
        assert!(contents.contains("B"));
        assert!(contents.contains("C"));
    }

    #[test]
    fn test_sorting() {
        let file = create_test_csv();
        let mut plugin = DegreePlugin::new();
        plugin.input(file.path()).unwrap();
        plugin.run();

        let output_file = NamedTempFile::new().unwrap();
        plugin.output(output_file.path()).unwrap();

        let contents = std::fs::read_to_string(output_file.path()).unwrap();
        let lines: Vec<&str> = contents.lines().collect();

        // First data line should be A (highest centrality)
        assert!(lines[1].starts_with("A"));
        // Second should be B
        assert!(lines[2].starts_with("B"));
        // Third should be C (lowest centrality)
        assert!(lines[3].starts_with("C"));
    }

    #[test]
    fn test_negative_weights() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "\"\",\"X\",\"Y\"").unwrap();
        writeln!(file, "\"X\",1,-0.5").unwrap();
        writeln!(file, "\"Y\",-0.5,1").unwrap();

        let mut plugin = DegreePlugin::new();
        plugin.input(file.path()).unwrap();
        plugin.run();

        let centrality = plugin.get_centrality();
        // X: 1 + (-0.5) = 0.5
        // Y: (-0.5) + 1 = 0.5
        assert!((centrality[0] - 0.5).abs() < 0.01);
        assert!((centrality[1] - 0.5).abs() < 0.01);
    }
}
