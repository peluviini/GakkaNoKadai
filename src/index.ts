
const sheetUrl = "https://docs.google.com/spreadsheets/d/12KPpMY7eV-cPQyw1QVUG0R_QObp-cpFYQhn_H8V2V_I/edit?gid=0#gid=0";
const stateCell = "H1";
type sheet = {
  timestamp: Date,
  temperature: string,
  pressure: string,
  illuminance: string,
};


const webhookUrl = "https://discord.com/api/webhooks/1392685132707528736/gjHwgfq3JttlRXPBOBvN0nqJNToUlo3RwN1yIiuz0GTWXuP8ir7Iq7CA3Ax1XhFVODBt";
type headerJson = {
  method: "get" | "delete" | "patch" | "post" | "put",
  contentType: "application/json",
  payload: string
};
const f_headerJson = (m: "post" | "get", p: any): headerJson => {
  return {
    method: m,
    contentType: "application/json",
    payload: JSON.stringify(p)
  };
};
function sendWebhook(message: string) {
  const payload = {
    content: message
  };
  const header = f_headerJson("post", payload);
  UrlFetchApp.fetch(webhookUrl, header);
}



function doGet(e: GoogleAppsScript.Events.DoGet) {
  const ss = SpreadsheetApp.openByUrl(sheetUrl)
  const sheet = ss.getSheets()[0];

  const params: sheet = {
    timestamp: new Date(),
    temperature: e.parameter.temperature || "",
    pressure: e.parameter.pressure || "",
    illuminance: e.parameter.illuminance || "",
  };
  sheet.appendRow(Object.values(params));

  return HtmlService.createTemplateFromFile("index").evaluate();
}

function doPost (e: GoogleAppsScript.Events.DoPost) {
}

function toggleIsOn ()
{
  const ss = SpreadsheetApp.openByUrl(sheetUrl);
  const sheet = ss.getSheets()[0];
  const cell = sheet.getRange(stateCell);
  const currentState = cell.getValue();

  cell.setValue(currentState === "ON" ? "OFF" : "ON");

  sendWebhook(currentState);
}