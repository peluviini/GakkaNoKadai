"use strict";
var sheetUrl = "https://docs.google.com/spreadsheets/d/12KPpMY7eV-cPQyw1QVUG0R_QObp-cpFYQhn_H8V2V_I/edit?gid=0#gid=0";
var stateCell = "H1";
var webhookUrl = "https://discord.com/api/webhooks/1392685132707528736/gjHwgfq3JttlRXPBOBvN0nqJNToUlo3RwN1yIiuz0GTWXuP8ir7Iq7CA3Ax1XhFVODBt";
var f_headerJson = function (m, p) {
    return {
        method: m,
        contentType: "application/json",
        payload: JSON.stringify(p)
    };
};
function sendWebhook(message) {
    var payload = {
        content: message
    };
    var header = f_headerJson("post", payload);
    UrlFetchApp.fetch(webhookUrl, header);
}
var angleCell = "I1";
function doGet( /*e: GoogleAppsScript.Events.DoGet*/) {
    /*const ss = SpreadsheetApp.openByUrl(sheetUrl)
    const sheet = ss.getSheets()[0];
  
    const params: sheet = {
      timestamp: new Date(),
      temperature: e.parameter.temperature || "",
      pressure: e.parameter.pressure || "",
      illuminance: e.parameter.illuminance || "",
    };
    sheet.appendRow(Object.values(params));*/
    return HtmlService.createTemplateFromFile("index").evaluate();
}
function doPost(e) {
    try {
        var body = JSON.parse(e.postData.contents);
        var ss = SpreadsheetApp.openByUrl(sheetUrl);
        var sheet = ss.getSheets()[0];
        if (body.method === "post") {
            if (body.content === "sensor") {
                var cell = sheet.getRange(angleCell);
                cell.setValue(body.params.angle);
            }
        }
    }
    catch (e) { }
}
function toggleIsOn() {
    var ss = SpreadsheetApp.openByUrl(sheetUrl);
    var sheet = ss.getSheets()[0];
    var cell = sheet.getRange(stateCell);
    var currentState = cell.getValue();
    cell.setValue(currentState === "ON" ? "OFF" : "ON");
    sendWebhook(currentState);
}
