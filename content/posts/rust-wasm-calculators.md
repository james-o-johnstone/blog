---
title: "Running Calculators Implemented In Rust Using WebAssembly"
date: 2024-01-28T14:58:22Z
summary: "Some calculators I use for running, in the format I wanted on a single page. The underlying calculations are performed in Rust."
draft: false
---


{{< rawhtml >}}
<head>
<script src="/assets/bootstrap.js"></script>
<style>
table.hidden {
  display: none;
}
</style>
</head>

<div id="menu" style="margin:auto">
  <li class="active" id=paceNav>Pace </li>
  <li id=distanceNav>Distance</li>
</div> 
<p>

<div>
<table id=paceTable>
  <tr>
    <td>Pace</td>
    <td><input id="paceCalcPace" placeholder="mm:ss" type="string" name="pace" size="5"></td>
    <td>
        <select name="unit" id="paceCalcUnit">
          <option value="kms">per km</option>
          <option value="miles">per mile</option>
        </select>
    </td>
  </tr>
  <tr>
    <td>Result</td>
    <td id="paceCalcResult"></td>
  </tr>
<td><input id="calculatePace" type="button" class="button" value="Calculate" style="vertical-align:bottom;margin:0"></td>
</table> 
</div>

<table class="hidden" id=distanceTable>
  <tr>
    <td>Pace</td>
    <td><input id="distanceCalcPace" placeholder="mm:ss" type="string" name="pace" size="5"></td>
    <td>
        <select name="unit" id="distanceCalcUnit">
          <option value="km">per km</option>
          <option value="miles">per mile</option>
        </select>
    </td>
  </tr>
  <tr>
    <td>Time</td>
    <td><input id="distanceCalcTime" placeholder="hh:mm:ss" type="string" name="time" size="8"></td>
  </tr>
  <tr>
    <td>Result</td>
    <td id="distanceCalcResult"></td>
  </tr>
<td><input id="calculateDistance" type="button" class="button" value="Calculate" style="vertical-align:bottom;margin:0"></td>
</table> 
{{< /rawhtml >}}

Some calculators I use for running, available in the format I wanted all in a single page. I wanted to try out [rust-wasm](https://rustwasm.github.io/book/) for this, so the underlying calculations are performed in Rust (obviously this is overkill but it was a fun experiment).

Source code can be found [here](https://github.com/james-o-johnstone/blog/tree/master/src)

To get started with rust-wasm I followed the tutorial which can be found [here](https://rustwasm.github.io/book/game-of-life/setup.html) and I used [this guide](https://dev.to/tegandbiscuits/building-a-static-site-with-hugo-and-webpack-pd3) to bundle the JS with Hugo using webpack.

In order to include the HTML and js in this hugo blog post I used a [rawhtml shortcode](https://anaulin.org/blog/hugo-raw-html-shortcode/) `{{ <rawhtml> }}{{ </rawhtml> }}`.
