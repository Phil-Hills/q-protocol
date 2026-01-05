/**
 * Q-PROTOCOL :: Z-ORDER ENCODER
 * Author: Phil Hills (Systems Architect)
 */

module.exports = {
    /**
     * Encodes two 16-bit integers into a 32-bit Morton code (Z-order curve key).
     * @param {number} x - The x coordinate (0-65535)
     * @param {number} y - The y coordinate (0-65535)
     * @returns {number} The Z-order curve value.
     */
    encode: function (x, y) {
        // Basic Morton Code implementation
        x &= 0x0000ffff;
        x = (x | (x << 8)) & 0x00ff00ff;
        x = (x | (x << 4)) & 0x0f0f0f0f;
        x = (x | (x << 2)) & 0x33333333;
        x = (x | (x << 1)) & 0x55555555;

        y &= 0x0000ffff;
        y = (y | (y << 8)) & 0x00ff00ff;
        y = (y | (y << 4)) & 0x0f0f0f0f;
        y = (y | (y << 2)) & 0x33333333;
        y = (y | (y << 1)) & 0x55555555;

        return x | (y << 1);
    },

    /**
     * Decodes a 32-bit Morton code back into x and y coordinates.
     * @param {number} z - The Z-order curve value
     * @returns {Object} An object {x, y} containing the decoded coordinates.
     */
    decode: function (z) {
        function compact1By1(x) {
            x &= 0x55555555;
            x = (x ^ (x >> 1)) & 0x33333333;
            x = (x ^ (x >> 2)) & 0x0f0f0f0f;
            x = (x ^ (x >> 4)) & 0x00ff00ff;
            x = (x ^ (x >> 8)) & 0x0000ffff;
            return x;
        }
        return {
            x: compact1By1(z),
            y: compact1By1(z >> 1)
        };
    }
};
