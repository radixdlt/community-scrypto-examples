const express = require('express');
const router = express.Router();
const util = require('util');
const exec = util.promisify(require('child_process').exec);
const RADIX_TOKEN_ADDRESS = "030000000000000000000000000000000000000000000000000004";

let PACKAGE;
let GAMES = {};
let USERS = {};

;(async function() {
    const response = await publish_code();
    if (response?.error){ return console.error("There's been an error: ", response) }
    PACKAGE = response?.package;
}());

function getIo(req) { return req.app.get('socketio'); }

function ioMiddleWare(io) {
    io.on('connection', async (socket) => {
        await delay(1000);
        const account = await new_account();
        USERS[socket.id] = USERS[socket.id] || {};
        USERS[socket.id].socket = socket;
        USERS[socket.id] = {...USERS[socket.id], ...account};
        // console.log('a user connected', {USERS});

        socket.on('create-game', async ({name, bet}) => {
            if (!name || !bet) { return };
            const response = await create_game(name, bet);
            // console.log('creating game through socket: ', {response});
            io.emit("got-games", GAMES);
            socket.emit('new-account', {account, response});
        });
      
        socket.on('join-game', async (component, bet) => {
            USERS[socket.id] = {...USERS[socket.id], component, bet};
            const game = GAMES[component];
            // console.log(`user '${socket.id} joining game`, {game, user: USERS[socket.id], bet});
            const response = await join_game(account.key, account.account, component, game.bet);
            if (response.error){ console.error(response); }

            socket.join(component);
            // console.log("Response", {response});

            io.to(component).emit("server", {message: "User joined the room..."});

            const state = await check_state(component);
            io.to(component).emit("got-state", state);
        });

        socket.on('make-guess', async (guess) => {
            const user = USERS[socket.id];
            const response = await make_guess(user.key, user.account, user.component, guess);
            // console.log("Response", response, guess);
            socket.emit("server", {message: response});
            
            const state = await check_state(user.component);
            io.to(user.component).emit("got-state", state);
        });

        socket.on('withdraw-funds', async () => {
            const user = USERS[socket.id];
            const response = await withdraw_funds(user.key, user.account, user.component);
            // console.log("Response", response);
            socket.emit("server", {message: response});
            
            const state = await check_state(user.component);
            io.to(user.component).emit("got-state", state);
        });
        
        socket.on('disconnect', () => {
          console.log(`user '${socket.id} disconnected`);
          delete USERS[socket.id];
        });
        
        socket.emit("got-games", GAMES);
    });

    return function middleware(req, res, next) {
        next();
    }
}

    
/* GET home page. */
router.get('/', function(req, res, next) {
    const games = Object.keys(GAMES).map(k => GAMES[k]);
    console.log('games: ', {games});
    res.render('radix', { title: 'Radix Guess-It Game', games });
});

router.post('/new-account', async function(req, res, next) {
    const response = await new_account();
    if (response.error){ return res.status(500).json(response) }
    return res.json(response);
});

router.post('/login', async function(req, res, next) {
    const { public, private } = req.body;
    const response = await login_user(public, private);
    if (response.error){ return res.status(500).json(response) }
    return res.json(response.split("\n").slice(0,-1));
});

router.post('/create-game', async function(req, res, next) {
    const { name, bet } = req.body;
    const response = await create_game(name, bet);
    if (response.error){ return res.status(500).json(response) }

    const io = getIo(req);
    io.emit("got-games", GAMES);

    return res.json(response);
});

router.post('/join-game', async function(req, res, next) {
    const { public, private, component, bet } = req.body;
    const response = await join_game(public, private, component, bet);
    if (response.error){ return res.status(500).json(response) }

    return res.json(response);
});

router.post('/make-guess', async function(req, res, next) {
    const { public, private, component, guess } = req.body;
    const response = await make_guess(public, private, component, guess);
    if (response.error){ return res.status(500).json(response) }

    return res.json(response);
});

router.post('/check-state', async function(req, res, next) {
    const { component } = req.body;
    const response = await check_state(component);
    if (response.error){ return res.status(500).json(response) }

    return res.json(response);
});

router.post('/withdraw-funds', async function(req, res, next) {
    const { public, private, component } = req.body;
    const response = await withdraw_funds(public, private, component);
    if (response.error){ return res.status(500).json(response) }

    return res.json(response);
});

function getLogMessage(error) {
    const message = (error.stdout
        .split("\n").slice(-3, -1)
        .map(string => string.replaceAll(/└─ \[ERROR\]/gi, "").trim())[0] || "")
        .replaceAll(/Logs: 0|New Entities: 0/gi, "");
    const tx_status = error.stdout.split("\n").find(n => n.search(/Transaction Status:/gi) > -1);
    return message || tx_status;
}

async function publish_code() {
    try {
        exec("resim reset");
        const { stdout, stderr } = await exec("resim publish ..");
        const package = stdout.split(" ").pop().replace(/\n/gi, "");
        const hardError = stderr.search(/Finished release/gi) < 1;
        // console.log('{ stdout, stderr }: ', { stdout, stderr, package, hardError });

        if (hardError) { throw Error(stderr) }

        return {
            package,
        };
    } catch (e) {
        const logMessage = getLogMessage(e);
        return {error: {...e, message: logMessage || e.stderr}}
    }
}

async function new_account() {
    try {
        const { stdout, stderr } = await exec("resim new-account");
        const output = stdout.split('\n').map(string => string.replaceAll(/Account address:\s|Public key:\s/gi, ""));
        output.shift();
        output.pop()

        if (stderr) { throw Error(stderr) }

        return {
            account: output[0],
            key: output[1],
        };
    } catch (e) {
        const logMessage = getLogMessage(e);
        return {error: {...e, message: logMessage || e.stderr}}
    }
}

async function create_game(name, bet) {
    try {
        const { stdout, stderr } = await exec(`resim call-function ${PACKAGE} GuessIt create "${name}" ${bet}`);
        const [resourceDef, component] = stdout.split("\n").slice(-3, -1).map(string => string.replaceAll(/ResourceDef:\s|Component:\s|├─|└─/gi, "").trim());
        // console.log('stdout: ', {stdout, private, public, resourceDef, component});
        if (stderr) { throw Error(stderr) }

        // Store game ref
        const response = {
            resourceDef,
            component,
            name,
        };
        
        GAMES[component] = response;
        GAMES[component] = {...GAMES[component], bet}
        // console.log("Created game", {response, GAMES});
        return response;
    } catch (e) {
        const logMessage = getLogMessage(e);
        return {error: {...e, message: logMessage || e.stderr}}
    }
}

async function login_user(public, private) {
    try {
        const { stdout, stderr } = await exec(`resim set-default-account ${private} ${public}`);
        const isValid = stdout.search(/Default account updated!/gi) > -1;
        // console.log('stdout: ', {stdout, private, public, resourceDef, component});
        if (stderr || !isValid) { throw Error(stderr) }

        return Promise.resolve(`Account: ${private}\nPublic key: ${public}\nOutput: ${stdout}`);
    } catch (e) {
        const logMessage = getLogMessage(e);
        return {error: {...e, message: logMessage || e.stderr}}
    }
}

async function join_game(public, private, component, bet) {
    try {
        await login_user(public, private);
        await delay(1000);
        const payment = `${bet},${RADIX_TOKEN_ADDRESS}`;
        const { stdout, stderr } = await exec(`resim call-method ${component} join ${payment}`);
        const isValid = stdout.search(/Transaction Status: SUCCESS/gi) > -1;
        // console.log('stdout: ', {stdout, private, public, component});
        if (stderr || !isValid) { throw Error(stderr) }
        return stdout.split("\n");
    } catch (e) {
        const logMessage = getLogMessage(e);
        return {error: {...e, message: logMessage || e.stderr}}
    }
}

async function make_guess(public, private, component, guess) {
    try {
        await login_user(public, private);
        await delay(1000);
        const badge = `1,${GAMES[component]?.resourceDef}`;
        const { stdout, stderr } = await exec(`resim call-method ${component} make_guess ${guess} ${badge}`);
        const isValid = stdout.search(/Transaction Status: SUCCESS/gi) > -1;
        // console.log('stdout: ', {stdout, private, public, component, badge});
        if (stderr || !isValid) { throw Error(stderr) }
        return stdout.split("\n").slice(-5,-4).map(string => string.replaceAll(/├─|└─/gi, "").trim());
    } catch (e) {
        const logMessage = getLogMessage(e);
        return {error: {...e, message: logMessage || e.stderr}}
    }
}

async function check_state(component) {
    try {
        const { stdout, stderr } = await exec(`resim call-method ${component} state`);
        const isValid = stdout.search(/Transaction Status: SUCCESS/gi) > -1;
        // console.log('stdout: ', {stdout, private, public, component, badge});
        if (stderr || !isValid) { throw Error(stderr) }
        const response = stdout.split("\n").slice(-5,-4).map(string => string.replaceAll(/├─|└─/gi, "").trim());
        return JSON.parse(response?.[0].substr(1,response?.[0].length-2));
    } catch (e) {
        const logMessage = getLogMessage(e);
        return {error: {...e, message: logMessage || e.stderr}}
    }
}

async function withdraw_funds(public, private, component) {
    try {
        await login_user(public, private);
        const badge = `1,${GAMES[component]?.resourceDef}`;
        const { stdout, stderr } = await exec(`resim call-method ${component} withdraw_funds ${badge}`);
        const isValid = stdout.search(/Transaction Status: SUCCESS/gi) > -1;
        // console.log('stdout: ', {stdout, private, public, component, badge});
        if (stderr || !isValid) { throw Error(stderr) }
        return stdout.split("\n").slice(-5,-4).map(string => string.replaceAll(/├─|└─/gi, "").trim());
    } catch (e) {
        const logMessage = getLogMessage(e);
        return {error: {...e, message: logMessage || e.stderr}}
    }
}

module.exports = {radixRouter: router, ioMiddleWare};

function delay(time) {
    return new Promise(function(resolve) { 
        setTimeout(resolve, time)
    });
}