use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};


struct PriceEntry {
    data_item: String,
    resource_name: String,
    interval_num: i32,
    interval_start: String, // Change this later to be a timestamp
    interval_end: String,   // Same here
    price: f32,
}


#[tokio::main]
async fn main() {
    let resp = reqwest::get("http://oasis.caiso.com/oasisapi/SingleZip?queryname=PRC_INTVL_LMP&startdatetime=20250102T07:00-0000&enddatetime=20250102T08:00-0000&version=3&market_run_id=RTM&node=LOGANCIT_LNODED1").await.unwrap();

    let path = Path::new("./data.zip");
    let path1 = Path::new("./data/");
    let mut file = File::create(&path).unwrap();
    let data = resp.bytes().await.unwrap();
    file.write_all(&data).unwrap();

    let zipfile = File::open(path).unwrap();
    let mut zip = zip::ZipArchive::new(zipfile).unwrap();
    let filename: &str = zip.file_names().next().unwrap();
    let xml_path = Path::new("./data/").join(filename);
    zip.extract(path1).unwrap();

    let xml_file = File::open(xml_path).unwrap();
    let xml_file = BufReader::new(xml_file);

    let parser = EventReader::new(xml_file);

    let mut depth = 0;

    for i in parser {
        match i {
            Ok(XmlEvent::StartElement { name, ..}) => {
                println!("{:spaces$}-{n}", "", spaces = depth * 2, n = name.local_name);
                depth += 1
            }
            Ok(XmlEvent::EndElement { name }) => {
                depth -= 1;
                println!("{:spaces$}-{n}", "", spaces = depth * 2, n = name.local_name);
            }
            Ok(XmlEvent::Characters(data)) => {
                println!("Data: {}", data.escape_debug());
                if data == "RTM" {
                }
            }
            Err(i) => {
                eprintln!("Error: {i}");
                break;
            }
            _ => {}
        }
    }

}
