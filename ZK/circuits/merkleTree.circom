pragma circom 2.0.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";

// Computes Poseidon([left, right])
template HashLeftRight() {
    signal input left;
    signal input right;
    signal output hash;

    component hasher = Poseidon(2);
    hasher.inputs[0] <== left;
    hasher.inputs[1] <== right;
    hash <== hasher.out;
}

// Если s == 0, вернуть [in[0], in[1]]
// Если s == 1, вернуть [in[1], in[0]]
template DualMux() {
    signal input in[2];
    signal input s;
    signal output out[2];

    s * (1 - s) === 0;
    out[0] <== (in[1] - in[0]) * s + in[0];
    out[1] <== (in[0] - in[1]) * s + in[1];
}

// Проверяет корректность меркл-доказательства
template MerkleTreeChecker(levels) {
    signal input leaf;
    signal input root;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    component selectors[levels];
    component hashers[levels];

    // Массив для хранения значения "текущего хэша" на каждом уровне.
    // curs[0] = leaf,
    // curs[1] = хэш от (curs[0] + pathElement[0]),
    // ...
    // curs[levels] = итоговый корень, который сравниваем с `root`
    signal curs[levels + 1];
    curs[0] <== leaf;

    for (var i = 0; i < levels; i++) {
        selectors[i] = DualMux();
        selectors[i].in[0] <== curs[i];
        selectors[i].in[1] <== pathElements[i];
        selectors[i].s <== pathIndices[i];

        hashers[i] = HashLeftRight();
        hashers[i].left <== selectors[i].out[0];
        hashers[i].right <== selectors[i].out[1];

        // Записываем полученный хэш в curs[i+1]
        curs[i + 1] <== hashers[i].hash;
    }

    // Сравниваем root и curs[levels].
    root === curs[levels];
}
