"use strict";
var sheetUrl = "https://docs.google.com/spreadsheets/d/12KPpMY7eV-cPQyw1QVUG0R_QObp-cpFYQhn_H8V2V_I/edit?gid=0#gid=0";
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
var powerOnCell = "H2";
var targetTempCell = "I2";
var targetAngleCell = "J2";
function doGet(e) {
    if (e.parameter.param === "html") {
        return HtmlService.createTemplateFromFile("index").evaluate();
    }
    else if (e.parameter.param === "receive") {
        var ss = SpreadsheetApp.openByUrl(sheetUrl);
        var sheet = ss.getSheets()[0];
        var power = sheet.getRange(powerOnCell);
        var targetTemp = sheet.getRange(targetTempCell);
        var angle = sheet.getRange(targetAngleCell);
        var params = {
            power: power.getValue(),
            targetTemp: targetTemp.getValue(),
            angle: angle.getValue(),
        };
        try {
            return ContentService.createTextOutput(JSON.stringify(params));
        }
        catch (err) {
            return ContentService.createTextOutput("errorrrr");
        }
    }
    else {
        return ContentService.createTextOutput("e");
    }
}
function doPost(e) {
    try {
        var body = JSON.parse(e.postData.contents);
        var ss = SpreadsheetApp.openByUrl(sheetUrl);
        var sheet = ss.getSheets()[0];
        if (body.method === "post") {
            if (body.content === "sensor") {
                var params = {
                    timestamp: new Date(),
                    temperature: "",
                    humidity: "",
                    wind_speed: "",
                    angle: body.params.angle || "0",
                };
                sheet.appendRow(Object.values(params));
                var angleCell = sheet.getRange(targetAngleCell);
                angleCell.setValue(body.params.angle || 0);
            }
            else if (body.content === "fan") {
                var params = {
                    timestamp: new Date(),
                    temperature: body.params.temperature || "",
                    humidity: body.params.humidity || "",
                    wind_speed: body.params.wind_speed || "",
                    angle: "",
                };
                sheet.appendRow(Object.values(params));
            }
        }
    }
    catch (e) { }
}
function toggleIsOn() {
    var ss = SpreadsheetApp.openByUrl(sheetUrl);
    var sheet = ss.getSheets()[0];
    var cell = sheet.getRange(powerOnCell);
    var currentState = cell.getValue();
    cell.setValue(currentState === "ON" ? "ON" : "OFF");
    sendWebhook(currentState);
}
