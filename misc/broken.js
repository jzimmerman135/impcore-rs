function add_one(x) { return x + 1; }
function add_two(x) { return add_one(add_one(x)); }

console.log(add_two(0));

function add_one(x) { return x + 100; }

console.log(add_two(0));
console.log(add_one(0));


