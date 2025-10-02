
const sheetUrl = "https://docs.google.com/spreadsheets/d/12KPpMY7eV-cPQyw1QVUG0R_QObp-cpFYQhn_H8V2V_I/edit?gid=0#gid=0";
type sheet = {
    timestamp: Date,
    temperature: string,
    humidity: string,
    wind_speed: string,
    angle: string,
};


const webhookUrl = "https://discord.com/api/webhooks/1392685132707528736/gjHwgfq3JttlRXPBOBvN0nqJNToUlo3RwN1yIiuz0GTWXuP8ir7Iq7CA3Ax1XhFVODBt";
type headerJsonDiscord = {
    method: "get" | "delete" | "patch" | "post" | "put",
    contentType: "application/json",
    payload: string
};
const f_headerJson = (m: "post" | "get", p: any): headerJsonDiscord => {
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

const powerOnCell = "H2";
const targetTempCell = "I2";
const TempCell = "J2";
const targetAngleCell = "K2";
type headerJnSend = {
    power: "ON" | "OFF",
    targetTemp: number,
    temperature: number,
    angle: number,
};
type headerJnSensor = {
    method: "post",
    content: "sensor" | "fan",
    params: {
        angle: number,
    }
};


function doGet(e: GoogleAppsScript.Events.DoGet) {

    if (e.parameter.param === "html") {
        return HtmlService.createTemplateFromFile("index").evaluate();
    } else if (e.parameter.param === "get") {
        const ss = SpreadsheetApp.openByUrl(sheetUrl)
        const sheet = ss.getSheets()[0];
        const power = sheet.getRange(powerOnCell);
        const targetTemp = sheet.getRange(targetTempCell);
        const temp = sheet.getRange(TempCell);
        const angle = sheet.getRange(targetAngleCell);

        let params: headerJnSend = {
            power: power.getValue(),
            targetTemp: targetTemp.getValue(),
            temperature: temp.getValue(),
            angle: angle.getValue(),
        }

        return params;
    } else {}

    return HtmlService.createTemplateFromFile("index").evaluate();
}

function doPost (e: GoogleAppsScript.Events.DoPost) {
    try {
        let body: headerJnSensor = JSON.parse(e.postData.contents);
        const ss = SpreadsheetApp.openByUrl(sheetUrl);
        const sheet = ss.getSheets()[0];

        if (body.method === "post") {
            if (body.content === "sensor") {
                const params: sheet = {
                    timestamp: new Date(),
                    temperature: e.parameter.temperature || "",
                    humidity: e.parameter.humidity || "",
                    wind_speed: e.parameter.wind_speed || "",
                    angle: e.parameter.angle || "",
                };
                sheet.appendRow(Object.values(params));
            }
        }
    } catch (e: any) {}
}

function toggleIsOn () {
    const ss = SpreadsheetApp.openByUrl(sheetUrl);
    const sheet = ss.getSheets()[0];
    const cell = sheet.getRange(powerOnCell);
    const currentState = cell.getValue();

    cell.setValue(currentState === "ON" ? "OFF" : "ON");

    sendWebhook(currentState);
}