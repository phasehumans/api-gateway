class BankAccount{
    constructor(accnum, balance, name){
        this.accnum= accnum
        this.balance= balance
        this.name= name
        // this.rs= rs
    }

    deposit(rs) {
        this.balance= this.balance + rs
        console.log(`Deposit ${rs} and Balance is ${this.balance}`);
    }

    withDraw(rs){
        this.balance = this.balance - rs;
        console.log(`withdraw ${rs} and Balance is ${this.balance}`)
    }

    checkBal(){
        return `balance is ${this.balance}`
    }

    getDetails(){
        return `name: ${this.name} accnum: ${this.accnum}`
    }
}


const acc1= new BankAccount(213, 100, "Chetan")
const acc2= new BankAccount(214, 500, "Chaitanya")

console.log(acc1.getDetails())
console.log(acc2.getDetails())


acc1.deposit(400)
acc2.withDraw(499)


console.log(acc1.checkBal())
console.log(acc2.checkBal())