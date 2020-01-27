#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate rusqlite;

use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use rocket::request::Form;
use rusqlite::Connection;
use rusqlite::params;

#[derive(Serialize)]
struct TemplateContext {
	x: f64,
	y: f64,
	r1: f64,
	r2: f64,
	r3: f64,
}

#[derive(FromForm, Debug)]
struct Data {
	station: String,
	rssi: f64,
}

fn newton_raphson(x: f64, y: f64, r1: f64, r2: f64) -> (f64, f64) {
	// Initialisation
	let mut xk = 4.5;
	let mut yk = 3.0;
	// learning rate
	let eta = 0.2;

	// Variable d'écart
	let mut dx = xk - x;
	let mut dy = yk - y;
	let mut f0 = xk * xk + yk * yk - r1 * r1;
	let mut f1 = dx * dx + dy * dy - r2 * r2;
	
	for _ in 0..5 {
		let c = eta * 2.0 / (yk * x - xk * y);
		// x(k+1) = x(k) - J^(-1) * f(xk)
		xk = xk - c * (dy * f0 - yk * f1);
		yk = yk - c * (-dx * f0 + xk * f1);
		// println!("x/y: {:.4} {:.4}", xk, yk);

		dx = xk - x;
		dy = yk - y;

		f0 = xk * xk + yk * yk - r1 * r1;
		f1 = dx * dx + dy * dy - r2 * r2;
		// println!("eps: {:.4}", f0 * f0 + f1 * f1);
	}

	return (xk, yk);
}

fn get_position(r1: f64, r2: f64, r3: f64, eps: f64) -> (f64, f64) {
	// positions des stations
	let x1 = 2.0;
	let y1 = 1.0;
	let x2 = 13.0;
	let y2 = 6.0;
	let x3 = 12.0;
	let y3 = 1.0;

	// On choisit (x1, y1) en tant que reference
	let x2p = x2 - x1;
	let y2p = y2 - y1;
	let x3p = x3 - x1;
	let y3p = y3 - y1;

	// parametres de la droite
	let a = x2 - x1;
	let b = y1 - y2;
	let c = x1 * y2 - y1 * x2;

	let (x, y) = newton_raphson(x2p, y2p, r1, r2);
	
	let d = (a * x + b * y + c) / (a * a + b * b);
	let dnx = d * b;
	let dny = d * a;

	let xp = x + dnx;
	let yp = y + dny;

	// test si la position trouvee est bien a la distance r3 de (x3, y3)
	let a0 = x - x3p;
	let a1 = y - y3p;

	// println!("{:.4}", a0 * a0 + a1 * a1 - r3 * r3);
	if (a0 * a0 + a1 * a1 - r3 * r3).abs() < eps {
		return (x + x1, y + y1);
	} else if (a * xp + b * yp + c).abs() < eps {
		// On a atterri sur la droite, on recupere le symetrique 
		return (xp + dnx + x1, yp + dny + y1);
	} else {
		// On est pas parti dans le bon sens, on recupere le symetrique
		return (x - 2.0 * dnx + x1, y - 2.0 * dny + y1);
	}
}

// paramètres à modifier ou à entrainer avec un réseau de neurones
fn distance_3ac3(rssi: f64) -> f64 {
	return -0.35 * rssi - 29.0;
}

fn distance_3acb(rssi: f64) -> f64 {
	return -0.83 * rssi - 36.5;
}

fn distance_bf94(rssi: f64) -> f64 {
	return -1.34 * rssi - 70.67;
}

#[get("/")]
fn index() -> Template {
	// A: -102 -51 -57
	// B: -94  -55 -58
	// C: -102 -58 -57
	// G: -103 -51 -59
	
	let conn = Connection::open("data.db").expect("data.db cannot be found.");

	let r1:f64 = conn.prepare("SELECT DISTANCE FROM _3AC3 ORDER BY ID DESC LIMIT 1;").expect("cannot prepare")
					.query(params![]).expect("cannot query")
					.next().expect("no result").expect("cannot find element")
					.get(0).expect("cannot get the element");
	let r2:f64 = conn.prepare("SELECT DISTANCE FROM _3ACB ORDER BY ID DESC LIMIT 1;").expect("cannot prepare")
					.query(params![]).expect("cannot query")
					.next().expect("no result").expect("cannot find element")
					.get(0).expect("cannot get the element");
	let r3:f64 = conn.prepare("SELECT DISTANCE FROM _BF94 ORDER BY ID DESC LIMIT 1;").expect("cannot prepare")
					.query(params![]).expect("cannot query")
					.next().expect("no result").expect("cannot find element")
					.get(0).expect("cannot get the element");

	// eps = 25.0, autorisation de +/- 5m
	let (posx, posy) = get_position(r1, r2, r3, 25.0);

	let context = TemplateContext {
		x: posx,
		y: posy,
		r1: r1,
		r2: r2,
		r3: r3,
	};
	// RENDERING
	Template::render("index", &context)
}

#[post("/sigfox", format = "application/x-www-form-urlencoded", data= "<data>")]
fn data(data: Option<Form<Data>>) {
	if let Some(data) = data {
		// println!("{:?}", data);
		let conn = Connection::open("data.db").expect("data.db cannot be found.");

		match data.station.as_ref() {
			"3AC3" => {
					conn.execute("INSERT INTO _3AC3 (RSSI, DISTANCE) VALUES (?1, ?2)", params![data.rssi, distance_3ac3(data.rssi)]).unwrap();
			}
			"3ACB" => {
					conn.execute("INSERT INTO _3ACB (RSSI, DISTANCE) VALUES (?1, ?2)", params![data.rssi, distance_3acb(data.rssi)]).unwrap();
			}
			"BF94" => {
					conn.execute("INSERT INTO _BF94 (RSSI, DISTANCE) VALUES (?1, ?2)", params![data.rssi, distance_bf94(data.rssi)]).unwrap();
			}
			_ => {}
		}
	}
}

fn rocket() -> rocket::Rocket {
	rocket::ignite()
		.mount("/", routes![index, data])
		.mount("/image", StaticFiles::from("./image")) 
		.attach(Template::fairing())
}

fn main() {
	rocket().launch();
}