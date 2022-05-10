// normal:

function namedFn() {beepBoop();}

const beepBoop = function namedFnExpr() {anonFn();};

const anonFn = function () {arrowFn();};

const arrowFn = () => { Klass.staticMethod();};

class Klass {
    static staticMethod() {new Klass();}

    constructor() {this.classMethod();}

    classMethod() {this.#privateMethod();}

    #privateMethod() {this.prototypeMethod();}
}

Klass/*foo*/.  prototype // comment
. prototypeMethod = () => {globalAssign();};

globalAssign = () => {obj.objectLiteralMethod();};

let obj = {
    objectLiteralMethod() {obj.objectLiteralAnon();},
    objectLiteralAnon: () => {throw new Error();},

};

// async
async function asyncNamedFn() {await asyncArrowFn();}
const asyncArrowFn = async () => {await AsyncKlass.asyncStaticMethod();};

class AsyncKlass {
    static async asyncStaticMethod() {
        let k = new AsyncKlass();
        await k.asyncClassMethod();
    }
    async asyncClassMethod() {await this.#privateAsyncMethod();}
    async #privateAsyncMethod() {await this.asyncProtoMethod();}
}

AsyncKlass.prototype.asyncProtoMethod = async function() {await asyncObj.asyncObjectLiteralMethod();};


let asyncObj = {
    async asyncObjectLiteralMethod() {await asyncObj.asyncObjectLiteralAnon();},
    asyncObjectLiteralAnon: async () => {throw new Error();},
};

// run and catch

// node has a default limit of 10
Error.stackTraceLimit = Infinity;

try {
    console.log("# sync stack trace");
    namedFn();
} catch (e) {
    console.log(e.stack);
    console.log();
}

async function asyncMain() {
    try {
        console.log("# async stack trace");
        await asyncNamedFn();
    } catch (e) {
        // lol safari, you try so hard to be smart, but are so stupid indeed
        let s = e.stack.toString().split("\n").join("\n\n");
        console.log(s);
        console.log();
    }
}
asyncMain();
