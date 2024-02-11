import * as wasm from 'converters';

const paceNav = document.getElementById('paceNav');
const distanceNav = document.getElementById('distanceNav');
const paceTable = document.getElementById('paceTable');
const distanceTable = document.getElementById('distanceTable');

paceNav.addEventListener('click', event => {
  paceNav.classList.add('active');
  distanceNav.classList.remove('active');
  paceTable.classList.remove('hidden');
  distanceTable.classList.add('hidden');
});

distanceNav.addEventListener('click', event => {
  paceNav.classList.remove('active');
  distanceNav.classList.add('active');
  paceTable.classList.add('hidden');
  distanceTable.classList.remove('hidden');
});

const calcPace = document.getElementById('calculatePace');
const paceCalcPace = document.getElementById('paceCalcPace');
const paceCalcUnit = document.getElementById('paceCalcUnit');
const paceCalcResult = document.getElementById('paceCalcResult');

(function() {
    var originalTextColour = paceCalcResult.style.color
    calcPace.addEventListener('click', event => {
      try {
          let pace = wasm.convert_pace(paceCalcPace.value, paceCalcUnit.value);
          paceCalcResult.style.color = originalTextColour
          paceCalcResult.textContent = pace;
      } catch(error) {
          paceCalcResult.style.color = "red";
          paceCalcResult.textContent = error;
      }
    });
})()

const calcDistance = document.getElementById('calculateDistance');
const distanceCalcPace = document.getElementById('distanceCalcPace');
const distanceCalcUnit = document.getElementById('distanceCalcUnit');
const distanceCalcResult = document.getElementById('distanceCalcResult');

(function() {
    var originalTextColour = distanceCalcResult.style.color
    calcDistance.addEventListener('click', event => {
      try {
          let distance = wasm.calculate_distance(distanceCalcTime.value, distanceCalcPace.value, distanceCalcUnit.value);
          distanceCalcResult.style.color = originalTextColour
          distanceCalcResult.textContent = distance;
      } catch(error) {
          distanceCalcResult.style.color = "red";
          distanceCalcResult.textContent = error;
      }
    });
})()

//wasm.greet('James');

//console.log('hello world');
