const express = require('express');
const router = express.Router();
const util = require('util');
const exec = util.promisify(require('child_process').exec);

let PACKAGE;
let GAMES = {};

;(async function() {
    const response = await publish_code();
    if (response?.error){ return console.error("There's been an error: ", response) }
    PACKAGE = response?.package;
    // console.log("Response", {PACKAGE});
}());

/* GET home page. */
router.get('/', function(req, res, next) {
  res.render('radix', { title: 'Radix Guess-It Game', games: Object.keys(GAMES).map(k => GAMES[k]) });
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
    const { public, private } = req.body;
    const response = await create_game(public, private);
    if (response.error){ return res.status(500).json(response) }
    // Store game ref
    GAMES[response?.component] = response;

    return res.json(response);
});

router.post('/join-game', async function(req, res, next) {
    const { public, private, component } = req.body;
    const response = await join_game(public, private, component);
    if (response.error){ return res.status(500).json(response) }

    return res.json(response);
});

router.post('/make-guess', async function(req, res, next) {
    const { public, private, component, guess } = req.body;
    const response = await make_guess(public, private, component, guess);
    if (response.error){ return res.status(500).json(response) }

    return res.json(response);
});

module.exports = router;

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
        return {error: e}
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
        return {error: e}
    }
}

async function create_game(public, private) {
    try {
        const { stdout, stderr } = await exec(`resim call-function ${PACKAGE} GuessIt create`);
        const [resourceDef, component] = stdout.split("\n").slice(-3, -1).map(string => string.replaceAll(/ResourceDef:\s|Component:\s|├─|└─/gi, "").trim());
        // console.log('stdout: ', {stdout, private, public, resourceDef, component});
        if (stderr) { throw Error(stderr) }

        return {
            resourceDef,
            component,
        };
    } catch (e) {
        return {error: e}
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
        return {error: e}
    }
}

async function join_game(public, private, component) {
    try {
        await login_user(public, private);
        const { stdout, stderr } = await exec(`resim call-method ${component} join`);
        const isValid = stdout.search(/Transaction Status: SUCCESS/gi) > -1;
        // console.log('stdout: ', {stdout, private, public, component});
        if (stderr || !isValid) { throw Error(stderr) }
        return stdout.split("\n");
    } catch (e) {
        return {error: e}
    }
}

async function make_guess(public, private, component, guess) {
    try {
        await login_user(public, private);
        const badge = `1,${GAMES[component]?.resourceDef}`;
        const { stdout, stderr } = await exec(`resim call-method ${component} make_guess ${guess} ${badge}`);
        const isValid = stdout.search(/Transaction Status: SUCCESS/gi) > -1;
        // console.log('stdout: ', {stdout, private, public, component, badge});
        if (stderr || !isValid) { throw Error(stderr) }
        return stdout.split("\n").slice(-5,-4).map(string => string.replaceAll(/├─|└─/gi, "").trim());
    } catch (e) {
        return {error: e}
    }
}