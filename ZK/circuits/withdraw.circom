pragma circom 2.0.0;

include "../../node_modules/circomlib/circuits/bitify.circom";
include "../../node_modules/circomlib/circuits/pedersen.circom";
include "merkleTree.circom";

// Вычисляет Pedersen(nullifier + secret)
template CommitmentHasher() {
    signal input nullifier;
    signal input secret;
    signal output commitment;
    signal output nullifierHash;

    component commitmentHasher = Pedersen(496);
    component nullifierHasher = Pedersen(248);

    component nullifierBits = Num2Bits(248);
    component secretBits = Num2Bits(248);

    // Разбиваем nullifier и secret на биты
    nullifierBits.in <== nullifier;
    secretBits.in <== secret;

    // Заполняем входы для Pedersen-хэшеров
    for (var i = 0; i < 248; i++) {
        nullifierHasher.in[i] <== nullifierBits.out[i];
        commitmentHasher.in[i] <== nullifierBits.out[i];

        // В commitmentHasher кладём и биты secret
        commitmentHasher.in[i + 248] <== secretBits.out[i];
    }

    // Результат хэширования
    commitment <== commitmentHasher.out[0];
    nullifierHash <== nullifierHasher.out[0];
}

// Проверяет, что commitment, соответствующий (nullifier, secret),
// действительно содержится в заданном меркл-дереве с корнем `root`.
template Withdraw(levels) {
    // Все эти сигналы – входы. В circom2 они по умолчанию приватные,
    // если не сделаны публичными в main.
    signal input root;
    signal input nullifierHash;
    signal input recipient; // не участвует в вычислениях напрямую
    signal input token;     // не участвует в вычислениях напрямую

    signal input nullifier;
    signal input secret;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    // Хешируем nullifier+secret, проверяем, что nullifierHash совпадает
    component hasher = CommitmentHasher();
    hasher.nullifier <== nullifier;
    hasher.secret <== secret;

    // Оператор === добавляет constraint (hasher.nullifierHash == nullifierHash)
    hasher.nullifierHash === nullifierHash;

    // Проверяем inclusion proof в меркл-дереве
    component tree = MerkleTreeChecker(levels);
    tree.leaf <== hasher.commitment;
    tree.root <== root;

    for (var i = 0; i < levels; i++) {
        tree.pathElements[i] <== pathElements[i];
        tree.pathIndices[i] <== pathIndices[i];
    }

    // «Фиктивные» квадраты, чтобы не дать оптимизатору удалить recipient или token
    signal recipientSquare;
    signal tokenSquare;

    recipientSquare <== recipient * recipient;
    tokenSquare <== token * token;
}

// Главный компонент, где указываем, какие входные сигналы делать публичными
component main {
    // Предположим, что хотим сделать публичными только root, nullifierHash, recipient, token
    public [root, nullifierHash, recipient, token]
} = Withdraw(2);
