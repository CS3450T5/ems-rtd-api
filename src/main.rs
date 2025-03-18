use std::os::unix::process;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};


#[derive(Debug)]
struct PriceEntry {
    data_item: String,
    resource_name: String,
    //interval_num: i32,
    //interval_start: String, // Change this later to be a timestamp
    //interval_end: String,   // Same here
    price: String //f32,
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

    let mut parser = EventReader::new(xml_file);

    let mut processing = true;

    while processing {
        let mut element = parser.next();
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
                    let mut itvl_num = "".to_string();
                    match parser.next() {
                        Ok(XmlEvent::Characters(val)) => {
                            itvl_num = val;
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
                    let mut value = "".to_string();
                    match parser.next() {
                        Ok(XmlEvent::Characters(val)) => {
                            value = val;
                        }
                        _ => {}
                    }

                    let thingy = PriceEntry{
                        data_item: prc_item,
                        resource_name: r_name,
                        price: value
                    };
                    println!("{:?}", thingy);
                }
            }
            _ => {}
        }
    }

    /*
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
                if data == "LMP_PRC" {
                    println!("Hey look thingy aaaaaaaaaaaaa")
                }
            }
            Err(i) => {
                eprintln!("Error: {i}");
                break;
            }
            _ => {}
        }
    }
    */

}
