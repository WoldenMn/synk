use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::{Duration, NaiveDate};

const HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE HealthData [
<!-- HealthKit Export Version: 14 -->
<!ELEMENT HealthData (ExportDate,Me,(Record|Correlation|Workout|ActivitySummary)*)>
<!ATTLIST HealthData
  locale CDATA #REQUIRED
>
<!ELEMENT Record (MetadataEntry*)>
<!ATTLIST Record
  type          CDATA #REQUIRED
  unit          CDATA #IMPLIED
  value         CDATA #IMPLIED
  sourceName    CDATA #REQUIRED
  startDate     CDATA #REQUIRED
  endDate       CDATA #REQUIRED
>
<!ELEMENT MetadataEntry EMPTY>
]>
<HealthData locale="fr_FR">
 <ExportDate value="2026-10-25 23:30:00 +0200"/>
 <Me HKCharacteristicTypeIdentifierDateOfBirth="1990-01-01"/>
"#;

const FOOTER: &str = "</HealthData>\n";

pub fn stamp(index: usize) -> String {
    let base = NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let at = base + Duration::seconds(index as i64 * 61);
    format!("{} +0200", at.format("%Y-%m-%d %H:%M:%S"))
}

pub fn record(kind: &str, unit: &str, value: &str, origin: &str, at: &str) -> String {
    format!(
        " <Record type=\"HKQuantityTypeIdentifier{kind}\" sourceName=\"{origin}\" sourceVersion=\"10.2\" unit=\"{unit}\" creationDate=\"{at}\" startDate=\"{at}\" endDate=\"{at}\" value=\"{value}\"/>\n"
    )
}

pub fn write_synthetic<W: Write>(w: &mut W, records: usize) -> std::io::Result<()> {
    w.write_all(HEADER.as_bytes())?;
    for i in 0..records {
        let at = stamp(i);
        let line = match i % 10 {
            0..=5 => record(
                "HeartRate",
                "count/min",
                &format!("{}", 55 + i % 40),
                "Apple Watch",
                &at,
            ),
            6 | 7 => record(
                "StepCount",
                "count",
                &format!("{}", 20 + i % 90),
                if i % 2 == 0 { "iPhone" } else { "Apple Watch" },
                &at,
            ),
            8 if i % 20 == 8 => record(
                "BodyMass",
                "lb",
                &format!("{:.1}", 150.0 + (i % 50) as f64 / 10.0),
                "Withings",
                &at,
            ),
            8 => record(
                "BodyMass",
                "kg",
                &format!("{:.1}", 70.0 + (i % 50) as f64 / 10.0),
                "Balance",
                &at,
            ),
            _ => record(
                "BloodGlucose",
                "mg/dL",
                &format!("{}", 80 + i % 60),
                "Libre",
                &at,
            ),
        };
        w.write_all(line.as_bytes())?;
    }
    w.write_all(FOOTER.as_bytes())
}

pub fn write_xml(dir: &Path, records: usize) -> PathBuf {
    let path = dir.join("export.xml");
    let mut w = BufWriter::new(File::create(&path).unwrap());
    write_synthetic(&mut w, records).unwrap();
    w.flush().unwrap();
    path
}

pub fn zip_of(dir: &Path, xml: &Path) -> PathBuf {
    zip_of_named(dir, xml, "export.zip")
}

pub fn small_export(dir: &Path, body: &str) -> PathBuf {
    let xml = dir.join("small.xml");
    let mut text = String::from(HEADER);
    text.push_str(body);
    text.push_str(FOOTER);
    std::fs::write(&xml, text).unwrap();
    zip_of_named(dir, &xml, "small.zip")
}

fn zip_of_named(dir: &Path, xml: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let mut zip = zip::ZipWriter::new(BufWriter::new(File::create(&path).unwrap()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("apple_health_export/export.xml", options)
        .unwrap();
    std::io::copy(&mut File::open(xml).unwrap(), &mut zip).unwrap();
    zip.finish().unwrap();
    path
}
