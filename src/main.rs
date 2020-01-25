#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate rusqlite;

use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use rocket::request::Form;
use rusqlite::Connection;
use rusqlite::NO_PARAMS;

#[derive(Serialize)]
struct TemplateContext {
	x: f64,
	y: f64,
}

#[derive(FromForm, Debug)]
struct Data {
	station: String,
	rssi: f64,
}

fn newton_raphson(x: f64, y: f64, r1: f64, r2: f64, eps: f64) -> (f64, f64) {
	// Initialisation
	let mut xk = 0.0;
	let mut yk = 0.0;

	// Variable d'écart
	let mut dx = xk - x;
	let mut dy = yk - y;
	let a0 = xk * xk + yk * yk - r1 * r1;
	let mut a1 = dx * dx + dy * dy - r2 * r2;
	
	while a0 * a0 + a1 * a1 > eps {
		let c = 2.0 / (yk * x - xk * y);

		// x(k+1) = x(k) - J^(-1) * f(xk)
		xk = xk - c * (dy * a0 - yk * a1);
		yk = yk - c * (-dx * a0 + xk * a1);

		dx = xk - x;
		dy = yk - y;

		a1 = dx * dx + dy * dy - r2 * r2;
	}

	return (xk, yk);
}

fn get_position(r1: f64, r2: f64, r3: f64, eps: f64) -> (f64, f64) {
	let x1 = 0.0;
	let y1 = 0.0;
	let x2 = 0.0;
	let y2 = 0.0;
	let x3 = 0.0;
	let y3 = 0.0;

	// On choisit (x1, y1) en tant que reference
	let x2p = x2 - x1;
	let y2p = y2 - y1;
	let x3p = x3 - x1;
	let y3p = y3 - y1;

	// parametres de la droite
	let a = x2 - x1;
	let b = y1 - y2;
	let c = x1 * y2 - y1 * x2;

	let pos = newton_raphson(x2p, y2p, r1, r2, eps);
	let x = pos.0;
	let y = pos.1;

	let d = (a * x + b * y + c) / (a * a + b * b);
	let dnx = d * b;
	let dny = d * a;

	let xp = x + dnx;
	let yp = y + dny;

	// test si la position trouvee est bien a la distance r3 de (x3, y3)
	let a0 = x - x3p;
	let a1 = y - y3p;
	if a0 * a0 + a1 * a1 - r3 * r3 < eps {
		return (x + x1, y + y1);
	} else if a * xp + b * yp + c < eps {
		// On a atterri sur la droite, on recupere le symetrique 
		return (xp + dnx + x1, yp + dny + y1);
	} else {
		// On est pas parti dans le bon sens, on recupere le symetrique
		return (x - 2.0 * dnx + x1, y - 2.0 * dny + y1);
	}
}

fn distance(rssi: f64) -> f64 {
	return -1.35 * rssi - 70.67;
}

#[get("/")]
fn index() -> Template {
	// CONTEXT
	// x : 0 ~ 750.0 == MIN ~ MAX
	// y : 0 ~ 285.0 == MIN ~ MAX
	// 50 pixels = 1 meter

	let context = TemplateContext {
		// x: 48.8534,
		// y: 2.3488,
		x: 50.0,//48.79217442020529,
		y: 50.0,//2.402670292855805,
	};
	// RENDERING
	Template::render("index", &context)
}

#[get("/hello/<name>/<age>")]
fn hello(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}


#[post("/sigfox", format = "application/x-www-form-urlencoded", data= "<data>")]
fn data(data: Option<Form<Data>>) {
	if let Some(data) = data {
		println!("{:?}", data);
		println!("{:?}", data.station);
		let conn = Connection::open("data.db").expect("data.db cannot be found.");

		match data.station.as_ref() {
			"3ACB" => {
					conn.execute(&format!("INSERT INTO _3ACB (RSSI, DISTANCE) VALUES ({}, {});", data.rssi, distance(data.rssi)), NO_PARAMS).unwrap();
			}
			"3AC3" => {
					conn.execute(&format!("INSERT INTO _3AC3 (RSSI, DISTANCE) VALUES ({}, {});", data.rssi, distance(data.rssi)), NO_PARAMS).unwrap();
			}
			"BF94" => {
					conn.execute(&format!("INSERT INTO _BF94 (RSSI, DISTANCE) VALUES ({}, {});", data.rssi, distance(data.rssi)), NO_PARAMS).unwrap();
			}
			_ => {}
		}
	}
}

fn rocket() -> rocket::Rocket {
	rocket::ignite()
		.mount("/", routes![index, data])
		.mount("/image", StaticFiles::from("/image")) 
		.attach(Template::fairing())
}

fn main() {
	rocket().launch();
}