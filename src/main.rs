use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use chrono::{DateTime, Local};

use xml::reader::{EventReader, XmlEvent};


#[derive(Debug)]
struct PriceEntry {
    data_item: String,
    resource_name: String,
    interval_num: i32,
    //interval_start: String, // Change this later to be a timestamp
    //interval_end: String,   // Same here
    price: f32,
}


#[tokio::main]
async fn main() {
    let curr: DateTime<Local> = Local::now();

    let formatted = format!("{}", curr.format("http://oasis.caiso.com/oasisapi/SingleZip?queryname=PRC_INTVL_LMP&startdatetime=%Y%m%dT00:00-0000&enddatetime=%Y%m%dT23:00-0000&version=3&market_run_id=RTM&node=LOGANCIT_LNODED1"));
    let resp = reqwest::get(formatted).await.unwrap();

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

    let mut parser = EventReader::new(xml_file);

    let mut processing = true;

    while processing {
        let element = parser.next();
        match element {
            Ok(XmlEvent::Characters(data)) => {
                if data == "LMP_PRC" {
                    let prc_item = data;
                    parser.next();
                    parser.next();
                    parser.next();
                    let mut r_name: String = "".to_string();
                    match parser.next() {
                        Ok(XmlEvent::Characters(next_data)) => {
                            r_name = next_data;
                        }
                        _ => {}
                    }
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    let mut itvl_num = 0;
                    match parser.next() {
                        Ok(XmlEvent::Characters(val)) => {
                            itvl_num = val.parse::<i32>().expect("Bad interval number");
                        }
                        _ => {}
                    }
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    parser.next();
                    let mut value = 0.0;
                    match parser.next() {
                        Ok(XmlEvent::Characters(val)) => {
                            value = val.parse::<f32>().expect("Bad data, price was not float");
                        }
                        _ => {}
                    }

                    let thingy = PriceEntry{
                        data_item: prc_item,
                        resource_name: r_name,
                        interval_num: itvl_num,
                        price: value
                    };
                    println!("{:?}", thingy);
                }
            }
            Ok(XmlEvent::EndDocument) => {
                processing = false;
            }
        
            _ => {}
        }
    }
}
