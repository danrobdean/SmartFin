/**
 * Maps keys to values, where each key maps to the equal or next-highest key in the keyset (if one exists).
 */
export default class NextMap {

    /**
     * The mapping of keys to values.
     */
    keyValueMap = {};

    /**
     * Add a value with the given key to the map.
     * @param key The key to store the key under.
     * @param value The value to add.
     */
    add(key, value) {
        this.keyValueMap[key.toString()] = value;
    }

    /**
     * Get the value stored at the next-highest (or equal) key.
     * @param key The key.
     */
    getNextValue(key) {
        var keys = Object.keys(this.keyValueMap).sort((a, b) => a - b);
        if (keys.length == 0) {
            throw "Could not find key for '" + key.toString() + "'. Keys: '" + keys + "'.";
        }

        var index = keys.findIndex(elem => key <= elem);
        if (index == -1) {
            throw "Could not find key for '" + key.toString() + "'. Keys: '" + keys + "'.";
        }

        return this.keyValueMap[keys[index].toString()];
    }

    /**
     * Get the value stored at the next-highest (or equal) key, or undefined if none exists.
     * @param key The key.
     */
    tryGetNextValue(key) {
        var keys = Object.keys(this.keyValueMap).sort((a, b) => a - b);
        if (keys.length == 0) {
            return undefined;
        }

        var index = keys.findIndex(elem => key <= elem);
        if (index == -1) {
            return undefined;
        }

        return this.keyValueMap[keys[index].toString()];
    }
}