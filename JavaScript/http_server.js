const express = require('express')

// import express from "express";
// if using import then change in package.json file

const app = express();

app.get("/", (req, res) => {
  res.send("Hello World");
});

app.listen(3000);