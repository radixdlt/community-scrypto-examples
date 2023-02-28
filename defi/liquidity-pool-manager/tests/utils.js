const fs = require("fs");
const os = require("os");
const path = require("path");

const envFilePath = path.resolve(__dirname, ".env");

// read .env file & convert to array
const readEnvVars = () => fs.readFileSync(envFilePath, "utf-8").split(os.EOL);

/**
 * Finds the key in .env files and returns the corresponding value
 *
 * @param {string} key Key to find
 * @returns {string|null} Value of the key
 */
const getEnvValue = (key) => {
  // find the line that contains the key (exact match)
  const matchedLine = readEnvVars().find((line) => line.split("=")[0] === key);
  // split the line (delimiter is '=') and return the item at index 2
  return matchedLine !== undefined ? matchedLine.split("=")[1] : null;
};

/**
 * Updates value for existing key or creates a new key=value line
 *
 * This function is a modified version of https://stackoverflow.com/a/65001580/3153583
 *
 * @param {string} key Key to update/insert
 * @param {string} value Value to update/insert
 */
const setEnvValue = (key, value) => {
  const envVars = readEnvVars();
  const targetLine = envVars.find((line) => line.split("=")[0] === key);
  if (targetLine !== undefined) {
    // update existing line
    const targetLineIndex = envVars.indexOf(targetLine);
    // replace the key/value with the new value
    envVars.splice(targetLineIndex, 1, `${key}="${value}"`);
  } else {
    // create new key value
    envVars.push(`${key}=${value}`);
  }
  // write everything back to the file system
  fs.writeFileSync(envFilePath, envVars.join(os.EOL));

  process.env[key] = value;
};

/**
 * Empty .env file
 */
const resetEnvs = (key, value) => {
  fs.writeFileSync(envFilePath, '');
  // process.env = {}
};

/**
* This function run a command and parse its output.
* The function runs the command passed as an argument either quietly (if isQuiet is true, the default value), or not. The function returns the result of running the command as an array of strings, which are the matched values of a regular expression pattern.
* The regular expression regexResim is defined at the top of the function. It looks for patterns that match specific text strings, such as "New Package", "Account component address", etc., and captures the following characters as a group (group 2).
* The matchAll method is used on the stdout property of the result of running the command, and for loop is used to iterate over all matches. For each match, the second captured group (the matched value) is pushed into the outputs array.
* Finally, the outputs array is returned as the result of the function.
 */
async function e(command, isQuiet = true) {
  const regexResim = /(New Package|Account component address|Private key|Public key|└─ Component|├─ Component|└─ Resource|├─ Resource|NFAddress): ([\d|A-Za-z|_|:|#]+)/gm


  let matches = (isQuiet ? await quiet(command) : await command).stdout.matchAll(regexResim)
  let outputs = []
  for (const match of matches) {
    outputs.push(match[2])
  }
  return outputs
}

module.exports = { getEnvValue, setEnvValue, resetEnvs, e }