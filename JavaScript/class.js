class Car{
    constructor(brand, price){
        this.brand= brand
        this.price= price
    }

    car_info(){
        console.log(`the brand name of car is ${this.brand} & it's price is ${this.price} Lakhs`);
        
    }

    priceGST(gst){
        let on_road= this.price + (this.price + gst / 100)
        console.log(`On road price of car is ${on_road} Lakhs`);
        
    }

}

const bmw= new Car('bmw', 70)

console.log(bmw.car_info());
console.log(bmw.brand)
console.log(bmw.priceGST(18))
// err: not identify fn ; soln --> args name and fn name should diffn ; price and priceGST
