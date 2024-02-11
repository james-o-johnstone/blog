const path = require('path');

module.exports = {
  entry: [path.resolve('src', 'js', 'bootstrap.js')],
  output: {
    path: path.resolve('static', 'assets'),
    publicPath: "/assets/",
    filename: "bootstrap.js",
  },
  mode: "development"
};
