use num::complex::Complex;
fn part_1() {
let mut a = Complex::new(4_f32, 2_f32);
let b = Complex::new(2_f32, 4_f32);
println!("({}) * ({}) = {}",a,b,a*b);
println!("({}) + ({}) = {}",a,b,a+b);
println!("|{}| = {}", a, Complex::norm(a));

println!("What happens to the size of a complew number when you repeatedly square it ?");

for i in 1..10{
    a = a * a;
    println!("Pow {} : {}", i, a);

}

}

fn escape_rule(cx: f32, cy: f32)->i32{
    let c = Complex::new(cx, cy);
    let mut z = Complex::new(0_f32,0_f32);
    let mut i = 0;
    let max_iterations = 1000;
    
    while Complex::norm(z) <= 2_f32 && i < max_iterations {
        z = z*z + c;
        i+=1;
    }
    i
}

fn get_mandelbraut_array(length: i32, width: i32) -> Vec<Vec<i32>> {
    let length = length as usize;
    let width = width as usize;

    let mut grid = Vec::with_capacity(length);

    for y in 0..length {
        let mut row = Vec::with_capacity(width);
        for x in 0..width {
            let re = -2.0_f32 + (x as f32) * (3.0_f32 / ((width - 1) as f32));     // real part: -2 to 1
            let im = -1.0_f32 + (y as f32) * (2.0_f32 / ((length - 1) as f32));    // imag part: -1 to 1
            row.push(escape_rule(re,im));
        }
        grid.push(row);
    }

    grid
}


fn main(){
    let grid = get_mandelbraut_array(24,100);
    for line in grid{
        for value in line {
           let value_repr = match value {
                0..=2 => ' ',
                3..=5 => '.',
                6..=10 => '*',
                11..=30 => '+',
                31..=100 => '#',
                _ => '%'
                };

            print!("{}",value_repr);
        }
        println!(""); }
}

