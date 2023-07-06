// If v is a number, rounds it to the specified precision.
// If v is not a number, returns it unchanged
function tryRound(v, precision) {
  if (typeof v == "number") {
    if (v % 1 == 0) {
      return v;
    } else {
      return v.toFixed(precision || 2);
    }
  } else {
    return v;
  }
}

// Returns n / d formatted as a percentage with the specified number
function toPercentage(n, d, decimals) {
  if (n === d) {
    return "100%";
  }
  return (
    (tryRound(n / d, decimals + 2 || 2) * 100)
      .toFixed(decimals || 0)
      .toString() + "%"
  );
}

// Given an object of the form {can: [list of rules], cannot: [list of rules]},
// Returns the percentage of total rules that can be derived
function getDerivability(o) {
  if (!o) {
    return "-";
  }
  let total = o.can.length + o.cannot.length;
  return toPercentage(o.can.length, total, 1);
}

// Pretty prints a list of rules using bidirectional arrows when possible
function formatRules(rules) {
  let bidir = [];
  if (!rules || rules.length == 0) {
    return "-";
  }
  rules.forEach((rule, i) => {
    let [left, right] = rule.split(" ==> ");
    if (rules.includes(`${right} ==> ${left}`)) {
      bidir.push(`${left} <=> ${right}`);
      rules.splice(i, 1);
    } else {
      bidir.push(`${left} ==> ${right}`);
    }
  });
  return bidir.join("<br />");
}

function reformat(keyMap, rows) {
  let tableData = [];
  rows.forEach((row) => {
    let newRow = {};
    Object.entries(keyMap).forEach(([key, f]) => {
      newRow[key] = tryRound(f(row));
    });
    tableData.push(newRow);
  });
  return tableData;
}

module.exports = {
  tryRound,
  formatRules,
  getDerivability,
  reformat,
};
