use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use chrono::NaiveDateTime;
use chrono::{DateTime, Local};
use xml::reader::{EventReader, XmlEvent};
use dotenv;
use mysql::*;
use mysql::prelude::*;
use std::time;


#[derive(Debug)]
struct PriceEntry {
    data_item: String,
    resource_name: String,
    interval_num: i32,
    interval_start: NaiveDateTime,
    interval_end: NaiveDateTime,
    price: f32,
}


#[tokio::main]
async fn main() {
    let db_pass = dotenv::var("DB_PASS").unwrap();
    let db_user = dotenv::var("DB_USER").unwrap();
    let db_name = dotenv::var("DB_NAME").unwrap();
    let db_host = dotenv::var("DB_HOST").unwrap();

    let curr: DateTime<Local> = Local::now();
    let mut data_points: Vec<PriceEntry> = Vec::new();

    let formatted = format!("{}", curr.format("http://oasis.caiso.com/oasisapi/SingleZip?queryname=PRC_INTVL_LMP&startdatetime=%Y%m%dT00:00-0600&enddatetime=%Y%m%dT23:00-0600&version=3&market_run_id=RTM&node=LOGANCIT_LNODED1"));
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

                    // Resource name
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

                    // Start interval time
                    let start_intvl: NaiveDateTime;
                    match parser.next() {
                        Ok(XmlEvent::Characters(val)) => {
                            start_intvl = DateTime::parse_from_str(&val, "%Y-%m-%dT%T%:z").unwrap().to_utc().naive_local().and_local_timezone(Local).unwrap().naive_local();
                        }
                        _ => {
                            return;
                        }
                    }
                    parser.next();
                    parser.next();
                    parser.next();

                    // End interval time
                    let mut end_intvl: NaiveDateTime;
                    match parser.next() {
                        Ok(XmlEvent::Characters(val)) => {
                            end_intvl = DateTime::parse_from_str(&val, "%Y-%m-%dT%T%:z").unwrap().to_utc().naive_local().and_local_timezone(Local).unwrap().naive_local();
                        }
                        _ => {
                            return;
                        }
                    }

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

                    let point = PriceEntry{
                        data_item: prc_item,
                        resource_name: r_name,
                        interval_num: itvl_num,
                        interval_start: start_intvl,
                        interval_end: end_intvl,
                        price: value
                    };
                    data_points.push(point);
                }
            }
            Ok(XmlEvent::EndDocument) => {
                processing = false;
            }
            _ => {}
        }
    }
    let url = Opts::from_url(format!("mysql://{}:{}@{}:3306/{}", db_user, db_pass, db_host, db_name).as_str()).unwrap();
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();

    conn.query_drop(r"CREATE TABLE IF NOT EXISTS energyprices (
            data_item text not null,
            resource_name text not null,
            interval_num int not null,
            interval_start datetime not null,
            interval_end datetime not null,
            price float not null
        )"
    ).unwrap();

    conn.exec_batch(
        r"INSERT INTO energyprices (data_item, resource_name, interval_num, interval_start, interval_end, price)
        VALUES (:data_item, :resource_name, :interval_num, :interval_start, :interval_end, :price)",
        data_points.iter().map(|p| params!{
            "data_item" => p.data_item.clone(),
            "resource_name" => p.resource_name.clone(),
            "interval_num" => p.interval_num,
            "interval_start" => p.interval_start,
            "interval_end" => p.interval_end,
            "price" => p.price
        })
    ).unwrap();


}
