use crate::Result;
use rust_xlsxwriter::{Color, Format, Workbook};
use std::path::Path;

pub fn export(sheets: &Path, inventory: &Path, output: &Path) -> Result<usize> {
    if output.exists() {
        return Err("workbook already exists: choose a new filename".into());
    }
    let mut workbook = Workbook::new();
    let header = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x173D52));
    let mut total = 0;
    for directory in [sheets, inventory] {
        let mut files: Vec<_> = std::fs::read_dir(directory)?
            .map(|e| e.map(|e| e.path()))
            .collect::<std::io::Result<_>>()?;
        files.retain(|p| p.extension().is_some_and(|e| e == "csv"));
        files.sort();
        for path in files {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("invalid worksheet filename")?;
            let mut csv = csv::Reader::from_path(&path)?;
            let headers = csv.headers()?.clone();
            let sheet = workbook.add_worksheet();
            sheet.set_name(name)?;
            sheet.set_freeze_panes(1, 0)?;
            for (col, value) in headers.iter().enumerate() {
                sheet.write_string_with_format(0, col as u16, value, &header)?;
                sheet.set_column_width(col as u16, if col == 0 { 28. } else { 40. })?;
            }
            let mut count = 0;
            for (row, record) in csv.records().enumerate() {
                if row >= 1_048_575 {
                    return Err(format!("{} exceeds Excel row limit", path.display()).into());
                }
                for (col, value) in record?.iter().enumerate() {
                    // All imported cells are text. Never evaluate formulas or hyperlinks from game content.
                    sheet.write_string((row + 1) as u32, col as u16, value)?;
                }
                count += 1;
            }
            if !headers.is_empty() {
                sheet.autofilter(0, 0, count as u32, (headers.len() - 1) as u16)?;
            }
            total += count;
        }
    }
    workbook.save(output)?;
    Ok(total)
}
