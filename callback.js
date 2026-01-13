function setTimeoutPromisified(ms) {
  return new Promise(function (resolve) {
    setTimeout(resolve, ms);
  });
}

// setTimeoutPromisified(500).then(console.log("done")) ---> this prints done and waits for 5sec
// pass fn in .then()

// still the cb hell
setTimeoutPromisified(3000).then(() => {
  console.log("after 3 sec");
  setTimeoutPromisified(5000).then(function () {
    //anynmous  fn
    console.log("after 5 sec");
    setTimeoutPromisified(7000).then(() => {
      console.log("after 7 sec");
    });
  });
});
