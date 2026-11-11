#![allow(dead_code)]

use std::path::{Path, PathBuf};

const FIT_EPOCH_OFFSET: i64 = 631_065_600;

const ENUM: u8 = 0x00;
const UINT8: u8 = 0x02;
const UINT16: u8 = 0x84;
const UINT32: u8 = 0x86;
const STRING: u8 = 0x07;

fn crc16(seed: u16, data: &[u8]) -> u16 {
    const TABLE: [u16; 16] = [
        0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401, 0xA001, 0x6C00, 0x7800,
        0xB401, 0x5000, 0x9C01, 0x8801, 0x4400,
    ];
    let mut crc = seed;
    for byte in data {
        let mut tmp = TABLE[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ TABLE[(byte & 0xF) as usize];
        tmp = TABLE[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ TABLE[((byte >> 4) & 0xF) as usize];
    }
    crc
}

pub fn fit_time(unix: i64) -> u32 {
    (unix - FIT_EPOCH_OFFSET) as u32
}

#[derive(Default)]
pub struct FitBuilder {
    data: Vec<u8>,
}

pub enum V {
    Enum(u8),
    U8(u8),
    U16(u16),
    U32(u32),
    Text(&'static str, usize),
}

impl V {
    fn base(&self) -> u8 {
        match self {
            V::Enum(_) => ENUM,
            V::U8(_) => UINT8,
            V::U16(_) => UINT16,
            V::U32(_) => UINT32,
            V::Text(..) => STRING,
        }
    }

    fn size(&self) -> u8 {
        match self {
            V::Enum(_) | V::U8(_) => 1,
            V::U16(_) => 2,
            V::U32(_) => 4,
            V::Text(_, n) => *n as u8,
        }
    }

    fn bytes(&self) -> Vec<u8> {
        match self {
            V::Enum(v) | V::U8(v) => vec![*v],
            V::U16(v) => v.to_le_bytes().to_vec(),
            V::U32(v) => v.to_le_bytes().to_vec(),
            V::Text(s, n) => {
                let mut b = s.as_bytes().to_vec();
                b.resize(*n, 0);
                b
            }
        }
    }
}

impl FitBuilder {
    pub fn message(&mut self, global: u16, fields: &[(u8, V)]) -> &mut Self {
        self.data.push(0x40);
        self.data.push(0);
        self.data.push(0);
        self.data.extend_from_slice(&global.to_le_bytes());
        self.data.push(fields.len() as u8);
        for (number, value) in fields {
            self.data
                .extend_from_slice(&[*number, value.size(), value.base()]);
        }
        self.data.push(0x00);
        for (_, value) in fields {
            self.data.extend(value.bytes());
        }
        self
    }

    pub fn finish(&self) -> Vec<u8> {
        let mut header = vec![14u8, 0x20];
        header.extend_from_slice(&2132u16.to_le_bytes());
        header.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        header.extend_from_slice(b".FIT");
        let header_crc = crc16(0, &header);
        header.extend_from_slice(&header_crc.to_le_bytes());
        let mut file = header;
        file.extend_from_slice(&self.data);
        let data_crc = crc16(0, &self.data);
        file.extend_from_slice(&data_crc.to_le_bytes());
        file
    }
}

const FILE_ID: u16 = 0;
const SESSION: u16 = 18;
const ACTIVITY: u16 = 34;
const WEIGHT_SCALE: u16 = 30;
const BLOOD_PRESSURE: u16 = 51;
const SLEEP_LEVEL: u16 = 275;

pub const RUN_START: i64 = 1_792_922_400;
pub const WEIGHT_AT: i64 = RUN_START - 9_000;
pub const NIGHT_START: i64 = RUN_START + 40_000;

pub fn sample_fit() -> Vec<u8> {
    let mut f = FitBuilder::default();
    f.message(
        FILE_ID,
        &[
            (0, V::Enum(4)),
            (1, V::U16(1)),
            (4, V::U32(fit_time(RUN_START))),
            (8, V::Text("Forerunner Test", 16)),
        ],
    );
    f.message(
        SESSION,
        &[
            (253, V::U32(fit_time(RUN_START + 1830))),
            (2, V::U32(fit_time(RUN_START))),
            (5, V::Enum(1)),
            (6, V::Enum(0)),
            (7, V::U32(1_830_000)),
            (8, V::U32(1_800_000)),
            (9, V::U32(512_345)),
            (11, V::U16(320)),
            (16, V::U8(148)),
            (17, V::U8(176)),
            (64, V::U8(102)),
            (22, V::U16(86)),
        ],
    );
    f.message(
        WEIGHT_SCALE,
        &[
            (253, V::U32(fit_time(WEIGHT_AT))),
            (0, V::U16(7_240)),
            (1, V::U16(1_850)),
            (2, V::U16(5_500)),
            (4, V::U16(310)),
            (5, V::U16(5_480)),
            (13, V::U16(231)),
        ],
    );
    f.message(
        BLOOD_PRESSURE,
        &[
            (253, V::U32(fit_time(WEIGHT_AT + 60))),
            (0, V::U16(121)),
            (1, V::U16(79)),
            (6, V::U8(64)),
        ],
    );
    let levels: [(i64, u8); 5] = [(0, 2), (10_800, 3), (16_200, 4), (19_800, 1), (21_000, 2)];
    for (offset, level) in levels {
        f.message(
            SLEEP_LEVEL,
            &[
                (253, V::U32(fit_time(NIGHT_START + offset))),
                (0, V::Enum(level)),
            ],
        );
    }
    f.message(ACTIVITY, &[(253, V::U32(fit_time(NIGHT_START + 25_800)))]);
    f.finish()
}

pub fn sample_gpx() -> String {
    let mut points = String::new();
    for i in 0..30 {
        let minute = i * 2;
        points += &format!(
            "  <trkpt lat=\"{:.5}\" lon=\"2.35000\"><ele>{}</ele><time>2026-10-25T10:{:02}:00Z</time><extensions><gpxtpx:TrackPointExtension><gpxtpx:hr>{}</gpxtpx:hr></gpxtpx:TrackPointExtension></extensions></trkpt>\n",
            48.85 + f64::from(i) * 0.0008,
            40 + (i % 7) * 4,
            minute % 60,
            120 + i
        );
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<gpx version=\"1.1\" creator=\"Montre synthetique\" xmlns=\"http://www.topografix.com/GPX/1/1\" xmlns:gpxtpx=\"http://www.garmin.com/xmlschemas/TrackPointExtension/v1\">\n <trk><name>Sortie inventee</name><type>cycling</type><trkseg>\n{points} </trkseg></trk>\n</gpx>\n"
    )
}

pub fn sample_tcx() -> String {
    let mut points = String::new();
    for i in 0..10 {
        points += &format!(
            "<Trackpoint><Time>2026-10-25T08:{:02}:00Z</Time><AltitudeMeters>{}</AltitudeMeters><DistanceMeters>{}</DistanceMeters><HeartRateBpm><Value>{}</Value></HeartRateBpm></Trackpoint>\n",
            i * 2,
            50 + i * 3,
            i * 600,
            110 + i * 4
        );
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<TrainingCenterDatabase xmlns=\"http://www.garmin.com/xmlschemas/TrainingCenterDatabase/v2\"><Activities><Activity Sport=\"Running\"><Id>2026-10-25T08:00:00Z</Id><Lap StartTime=\"2026-10-25T08:00:00Z\"><TotalTimeSeconds>1080</TotalTimeSeconds><DistanceMeters>5400</DistanceMeters><Calories>260</Calories><Track>\n{points}</Track></Lap><Creator><Name>Compteur synthetique</Name></Creator></Activity></Activities></TrainingCenterDatabase>\n"
    )
}

pub fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

pub fn sample_scale_csv() -> String {
    let mut text = String::from("Date;Heure;Poids (kg);Masse grasse (%);Tension sys;Tension dia\n");
    for d in 0..30 {
        text += &format!(
            "{:02}/10/2026;07:{:02};{},{};{},{};{};{}\n",
            d + 1,
            10 + d % 40,
            71 + d % 3,
            d % 10,
            18 + d % 2,
            d % 10,
            115 + d % 9,
            75 + d % 5
        );
    }
    text
}

pub fn sample_libreview_csv() -> String {
    let mut text = String::from("Patient report,Imprime le 2026-10-25,,,,,\nDevice,Serial Number,Device Timestamp,Record Type,Historic Glucose mg/dL,Scan Glucose mg/dL,Strip Glucose mg/dL\n");
    for i in 0..48 {
        text += &format!(
            "FreeStyle Libre,SYN001,{:02}-10-2026 {:02}:{:02},0,{},,\n",
            1 + i / 24,
            i % 24,
            (i * 7) % 60,
            90 + (i * 5) % 50
        );
    }
    text
}
