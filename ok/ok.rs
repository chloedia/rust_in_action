fn main(){
let needle = 0o204;
let haystack = [1,1,2,5,15,52,132,877,4140,21147];
for item in &haystack{
if needle == *item{
println!("The item is found, it is : {}",item);
}
}

}
