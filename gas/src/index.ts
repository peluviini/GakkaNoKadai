
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
const targetAngleCell = "J2";
type headerJnSend = {
    power: "ON" | "OFF",
    targetTemp: number,
    angle: number,
};
type headerJnSensor = {
    method: "post",
    content: "sensor" | "fan",
    params: {
        timestamp?: Date,
        temperature?: string,
        humidity?: string,
        wind_speed?: string,
        angle?: string,
    }
};


function doGet(e: GoogleAppsScript.Events.DoGet) {

    if (e.parameter.param === "html") {
        return HtmlService.createTemplateFromFile("index").evaluate();
    } else if (e.parameter.param === "recieve") {
        const ss = SpreadsheetApp.openByUrl(sheetUrl)
        const sheet = ss.getSheets()[0];
        const power = sheet.getRange(powerOnCell);
        const targetTemp = sheet.getRange(targetTempCell);
        const angle = sheet.getRange(targetAngleCell);

        let params: headerJnSend = {
            power: power.getValue(),
            targetTemp: targetTemp.getValue(),
            angle: angle.getValue(),
        }

        return JSON.stringify(params);
    } else {}
}

function doPost(e: GoogleAppsScript.Events.DoPost) {
    try {
        let body: headerJnSensor = JSON.parse(e.postData.contents);
        const ss = SpreadsheetApp.openByUrl(sheetUrl);
        const sheet = ss.getSheets()[0];

        if (body.method === "post") {
            if (body.content === "sensor") {
                const params: sheet = {
                    timestamp: new Date(),
                    temperature: "",
                    humidity: "",
                    wind_speed: "",
                    angle: body.params.angle || "0",
                };
                sheet.appendRow(Object.values(params));
                const angleCell = sheet.getRange(targetAngleCell);
                angleCell.setValue(body.params.angle || 0);
            } else if (body.content === "fan") {
                const params: sheet = {
                    timestamp: new Date(),
                    temperature: body.params.temperature || "",
                    humidity: body.params.humidity || "",
                    wind_speed: body.params.wind_speed || "",
                    angle: "",
                }
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