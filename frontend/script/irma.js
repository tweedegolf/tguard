const IrmaCore = require('@privacybydesign/irma-core');
const IrmaPopup = require('@privacybydesign/irma-popup');
const IrmaClient = require('@privacybydesign/irma-client');

window.startIrma = function (session) {
    const irma = new IrmaCore({ debugging: true, session });
    irma.use(IrmaClient);
    irma.use(IrmaPopup);

    return irma.start();
}
