use std::io::Write;

use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
use serde::{Deserialize, Serialize};

fn main() -> Result<(), Error> {
    convert("SUBTLEX-CH-WF.xlsx", "SUBTLEX-CH-WF", "word_freq.json")?;
    convert("SUBTLEX-CH-CHR.xlsx", "SUBTLEX-CH-CHR", "char_freq.json")?;
    Ok(())
}

fn convert(file: &str, workbook_name: &str, out_file: &str) -> Result<(), Error> {
    let mut workbook: Xlsx<_> = open_workbook(file)?;
    let range = workbook
        .worksheet_range(workbook_name)
        .ok_or(Error::Msg("Cannot find workbook"))??;

    let mut iter = RangeDeserializerBuilder::new().from_range(&range)?.skip(2);
    let mut fs = std::fs::File::create(out_file).unwrap();

    while let Some(Ok(result)) = iter.next() {
        let val: (String, u64, f64, f64, f64, f64, f64) = result;
        let row = FreqRow {
            text: val.0,
            count: val.1,
            count_per_million: val.2,
            log_count: val.3,
            cd: val.4,
            cd_percentage: val.5,
            log_cd: val.6,
        };
        fs.write_all(serde_json::to_string(&row).unwrap().as_bytes())
            .unwrap();
        fs.write_all(b"\n").unwrap();
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct FreqRow {
    text: String,
    count: u64,
    count_per_million: f64,
    log_count: f64,
    cd: f64,
    cd_percentage: f64,
    log_cd: f64,
}
