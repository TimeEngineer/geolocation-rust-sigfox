#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

use rocket_contrib::templates::Template;
use rocket::request::Form;

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

#[get("/")]
fn index() -> Template {
	// CONTEXT
	let context = TemplateContext {
		x: 48.8534,
		y: 2.3488,
	};
	// RENDERING
	Template::render("index", &context)
}

#[post("/sigfox", format = "application/x-www-form-urlencoded", data= "<data>")]
fn data(data: Option<Form<Data>>) {
	if let Some(data) = data {
		println!("{:?}", data);
	}
}

fn rocket() -> rocket::Rocket {
	rocket::ignite()
		.mount("/", routes![index, data])
		.attach(Template::fairing())
}

fn main() {
	rocket().launch();
}