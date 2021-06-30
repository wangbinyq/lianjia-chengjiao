#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate diesel;

mod schema;

use diesel::{prelude::*, Connection, SqliteConnection};
use schema::items;

use anyhow::Result;
use tokio::stream::StreamExt;
use voyager::{scraper::Selector, Collector, Crawler, CrawlerConfig, Response, Scraper};

const BASE_URL: &'static str = "https://sh.lianjia.com";
const INIT_URL: &'static str = "https://sh.lianjia.com/chengjiao/";

#[derive(Debug)]
enum State {
    Init,
    Quyu(String),           // 区域
    Page(String, String),   // 列表
    Detail(String, String), // 详情
}

#[derive(Debug, Serialize, Insertable)]
#[table_name = "items"]
struct Chengjiao {
    qu: String,
    zheng: String,
    url: String,

    name: String,
    huxing: String,
    floor: String,
    square: String,
    structs: String,
    inner_square: String,
    build_type: String,
    direction: String,
    build_year: String,
    build_decorate: String,
    build_struct: String,
    gongnuan: String,
    tihubi: String,
    dianti: String,

    quansu: String,
    guapai_time: String,
    yongtu: String,
    nianxian: String,
    fangquan: String,

    guapai_price: String,
    chengjiao_zhouqi: String,
    tiaojia: String,
    chengjiao_price: String,
    danjia: String,
}

fn insert_item(item: Chengjiao, db: &SqliteConnection) {
    use schema::items::dsl::*;

    if let Err(err) = diesel::insert_into(items).values(item).execute(db) {
        error!("insert into db error: {:?}", err);
    }
}

fn exist_url(_url: &str, db: &SqliteConnection) -> bool {
    use schema::items::dsl::*;

    if let Ok(count) = items.filter(url.eq(_url)).count().get_result::<i64>(db) {
        return count > 0;
    }
    return false;
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageData {
    total_page: i32,
    cur_page: i32,
}

struct LianjiaScraper {
    pub db: SqliteConnection,
}

impl LianjiaScraper {
    pub fn new(db: SqliteConnection) -> Self {
        Self { db }
    }
}

impl Scraper for LianjiaScraper {
    type Output = Chengjiao;
    type State = State;

    fn scrape(
        &mut self,
        response: Response<Self::State>,
        crawler: &mut Crawler<Self>,
    ) -> Result<Option<Self::Output>> {
        if let Some(ref state) = response.state {
            let html = response.html();
            match state {
                State::Init => {
                    let qu_selector = Selector::parse(".m-filter [data-role=\"ershoufang\"] a")
                        .expect("qu selector");
                    for qu in html.select(&qu_selector) {
                        if let Some(url) = qu.value().attr("href") {
                            let url = format!("{}{}", BASE_URL, url);
                            let qu = qu.text().collect::<String>();
                            info!("find qu: {}, url: {}", qu, url,);
                            crawler.visit_with_state(&url, State::Quyu(qu));
                        }
                    }
                }
                State::Quyu(qu) => {
                    let zheng_selector =
                        Selector::parse(".m-filter [data-role=\"ershoufang\"] div:nth-child(2) a")
                            .expect("zheng selector");
                    for zheng in html.select(&zheng_selector) {
                        if let Some(url) = zheng.value().attr("href") {
                            let url = format!("{}{}", BASE_URL, url);
                            let zheng = zheng.text().collect::<String>();
                            info!("find qu: {}, zheng: {}, url: {}, ", qu, zheng, url,);
                            crawler.visit_with_state(&url, State::Page(qu.to_string(), zheng));
                        }
                    }
                }
                State::Page(qu, zheng) => {
                    let page_selector =
                        Selector::parse("[comp-module=\"page\"]").expect("page selector");
                    let list_selector =
                        Selector::parse(".listContent .title a").expect("list selector");

                    for item in html.select(&list_selector) {
                        if let Some(url) = item.value().attr("href") {
                            if !exist_url(url, &self.db) {
                                crawler.visit_with_state(
                                    url,
                                    State::Detail(qu.clone(), zheng.clone()),
                                );
                            }
                        }
                    }

                    if let Some(page) = html.select(&page_selector).next() {
                        let page_data = page.value().attr("page-data");
                        let page_url = page.value().attr("page-url");

                        if let (Some(page_data), Some(page_url)) = (page_data, page_url) {
                            if let Some(page_data) =
                                serde_json::from_str::<PageData>(page_data).ok()
                            {
                                if page_data.cur_page < page_data.total_page {
                                    let next_page = page_data.cur_page + 1;
                                    let url = format!(
                                        "{}{}",
                                        BASE_URL,
                                        page_url.replace("{page}", &format!("{}", next_page))
                                    );
                                    info!("find qu: {}, zheng: {}, url: {}, ", qu, zheng, url,);
                                    crawler.visit_with_state(
                                        &url,
                                        State::Page(qu.to_string(), zheng.to_string()),
                                    )
                                }
                            }
                        }
                    }
                }
                State::Detail(qu, zheng) => {
                    let find_inner_text = |selector| {
                        html.select(&Selector::parse(selector).unwrap())
                            .next()
                            .map(|name| name.text().collect::<String>())
                            .unwrap_or_default()
                            .trim()
                            .to_string()
                    };

                    let parse_info = |t, n| {
                        html.select(
                            &Selector::parse(&format!(".{} .content li:nth-child({})", t, n))
                                .unwrap(),
                        )
                        .next()
                        .map(|el| el.text().skip(1).collect::<String>())
                        .unwrap_or_default()
                        .trim()
                        .to_string()
                    };
                    let parse_base = |n| parse_info("base", n);
                    let parse_transaction = |n| parse_info("transaction", n);

                    let name = find_inner_text(".index_h1");
                    info!("find item: {}", name);

                    return Ok(Some(Self::Output {
                        qu: qu.clone(),
                        zheng: zheng.clone(),
                        url: response.request_url.to_string(),

                        name,
                        guapai_price: find_inner_text(".info .msg span:nth-child(1) label"),
                        chengjiao_zhouqi: find_inner_text(".info .msg span:nth-child(2) label"),
                        tiaojia: find_inner_text(".info .msg span:nth-child(3) label"),
                        chengjiao_price: find_inner_text(".info .dealTotalPrice i"),
                        danjia: find_inner_text(".info .price b"),

                        huxing: parse_base(1),
                        floor: parse_base(2),
                        square: parse_base(3),
                        structs: parse_base(4),
                        inner_square: parse_base(5),
                        build_type: parse_base(6),
                        direction: parse_base(7),
                        build_year: parse_base(8),
                        build_decorate: parse_base(9),
                        build_struct: parse_base(10),
                        gongnuan: parse_base(11),
                        tihubi: parse_base(12),
                        dianti: parse_base(13),

                        quansu: parse_transaction(2),
                        guapai_time: parse_transaction(3),
                        yongtu: parse_transaction(4),
                        nianxian: parse_transaction(5),
                        fangquan: parse_transaction(6),
                    }));
                }
            }

            Ok(None)
        } else {
            Ok(None)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("database url");
    let db = diesel::SqliteConnection::establish(&database_url).expect("connect to database");

    let config = CrawlerConfig::default();
    let mut collector = Collector::new(LianjiaScraper::new(db), config);

    collector
        .crawler_mut()
        .visit_with_state(INIT_URL, State::Init);

    while let Some(Ok(item)) = collector.next().await {
        insert_item(item, &collector.scraper().db);
    }

    Ok(())
}
