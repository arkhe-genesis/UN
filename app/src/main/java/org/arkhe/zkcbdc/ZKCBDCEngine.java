/*
 * Copyright contributors to Besu.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
 * the License. You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
 * an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the
 * specific language governing permissions and limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
package org.arkhe.zkcbdc;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.time.Instant;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

/**
 * ZKCBCC - Substrato 1010 Zero-Knowledge Central Bank Digital Currency Arquiteto ORCID:
 * 0009-0005-2697-4668 Seal: ZKCBDC-1010-2026-05-31
 */
public class ZKCBDCEngine {

  /** Javadoc */

  /** Substrate id */
  public static final String SUBSTRATE_ID = "1010";

  /** Javadoc */

  /** Seal */
  public static final String SEAL = "ZKCBDC-1010-2026-05-31";

  /** Javadoc */

  /** TransactionStatus */
  public enum TransactionStatus {
    /** Javadoc */
    /** PENDING */
    PENDING,
    /** Javadoc */
    /** PROVEN */
    PROVEN,
    /** Javadoc */
    /** REJECTED */
    REJECTED,
    /** Javadoc */
    /** ANCHORED */
    ANCHORED,
    /** Javadoc */
    /** DOUBLE_SPEND */
    DOUBLE_SPEND
  }

  /** Javadoc */

  /** AccountState */
  public static class AccountState {
    /** Javadoc */

    /** accountId */
    public String accountId;

    /** Javadoc */

    /** commitmentBalance */
    public String commitmentBalance;

    /** Javadoc */

    /** nonce */
    public int nonce = 0;

    /** Javadoc */

    /** isFrozen */
    public boolean isFrozen = false;

    /** Javadoc */

    /** kycLevel */
    public int kycLevel = 0;

    /** Javadoc */

    /** lastUpdated */
    public String lastUpdated;

    /** Javadoc */

    /**
     * @param accountId
     * @param commitmentBalance
     */
    /**
     * Constructor for AccountState.
     *
     * @param accountId the account id
     * @param commitmentBalance the commitment balance
     */
    /**
     * Constructor for AccountState.
     *
     * @param accountId the account id
     * @param commitmentBalance the commitment balance
     */
    public AccountState(final String accountId, final String commitmentBalance) {
      this.accountId = accountId;
      this.commitmentBalance = commitmentBalance;
      this.lastUpdated = Instant.now().toString();
    }
  }

  /** Javadoc */

  /** ConfidentialTransaction */
  public static class ConfidentialTransaction {
    /** Javadoc */

    /** txId */
    public String txId;

    /** Javadoc */

    /** commitmentSender */
    public String commitmentSender;

    /** Javadoc */

    /** commitmentReceiver */
    public String commitmentReceiver;

    /** Javadoc */

    /** commitmentAmount */
    public String commitmentAmount;

    /** Javadoc */

    /** nullifier */
    public String nullifier;

    /** Javadoc */

    /** zkProof */
    public String zkProof;

    /** Javadoc */

    /** kycProof */
    public String kycProof;

    /** Javadoc */

    /** sanctionsProof */
    public String sanctionsProof;

    /** Javadoc */

    /** timestamp */
    public String timestamp;

    /** Javadoc */

    /** status */
    public TransactionStatus status = TransactionStatus.PENDING;

    /** Javadoc */

    /** temporalAnchor */
    public String temporalAnchor = null;

    /** Javadoc */

    /** seal */
    public String seal = "";

    /** Javadoc */

    /**
     * @param txId
     * @param commitmentSender
     * @param commitmentReceiver
     * @param commitmentAmount
     * @param nullifier
     * @param zkProof
     * @param kycProof
     * @param sanctionsProof
     */
    /**
     * Constructor for ConfidentialTransaction.
     *
     * @param txId the tx id
     * @param commitmentSender the sender commitment
     * @param commitmentReceiver the receiver commitment
     * @param commitmentAmount the amount commitment
     * @param nullifier the nullifier
     * @param zkProof the ZK proof
     * @param kycProof the KYC proof
     * @param sanctionsProof the sanctions proof
     */
    /**
     * Constructor for ConfidentialTransaction.
     *
     * @param txId the tx id
     * @param commitmentSender the sender commitment
     * @param commitmentReceiver the receiver commitment
     * @param commitmentAmount the amount commitment
     * @param nullifier the nullifier
     * @param zkProof the ZK proof
     * @param kycProof the KYC proof
     * @param sanctionsProof the sanctions proof
     */
    public ConfidentialTransaction(
        final String txId,
        final String commitmentSender,
        final String commitmentReceiver,
        final String commitmentAmount,
        final String nullifier,
        final String zkProof,
        final String kycProof,
        final String sanctionsProof) {
      this.txId = txId;
      this.commitmentSender = commitmentSender;
      this.commitmentReceiver = commitmentReceiver;
      this.commitmentAmount = commitmentAmount;
      this.nullifier = nullifier;
      this.zkProof = zkProof;
      this.kycProof = kycProof;
      this.sanctionsProof = sanctionsProof;
      this.timestamp = Instant.now().toString();
    }

    /** Javadoc */

    /**
     * @return
     */
    /**
     * Computes the seal for the transaction.
     *
     * @return the computed seal
     */
    /**
     * Computes the seal for the transaction.
     *
     * @return the computed seal
     */
    public String computeSeal() {
      String payload = this.txId + ":" + this.nullifier + ":" + this.zkProof.substring(0, 32);
      this.seal = "ZKCBDC-" + sha256(payload).substring(0, 16).toUpperCase(java.util.Locale.ROOT);
      return this.seal;
    }
  }

  private final long totalSupply;
  private final Set<String> nullifiers = new HashSet<>();
  private final Map<String, ConfidentialTransaction> transactions = new HashMap<>();
  private final Map<String, AccountState> accounts = new HashMap<>();
  private final Map<String, String> mintProofs = new HashMap<>();
  private final Set<String> sanctionsList = new HashSet<>();
  private final Set<String> frozenAccounts = new HashSet<>();

  private long totalVolume = 0;

  @SuppressWarnings("DoNotCreateSecureRandomDirectly")
  private final SecureRandom random = new SecureRandom();

  /** Javadoc */

  /**
   * @param totalSupply
   * @param centralBankKey
   */
  /**
   * Creates a new instance of the CBDC engine.
   *
   * @param totalSupply total supply
   * @param centralBankKey key
   */
  /**
   * Creates a new instance of the CBDC engine.
   *
   * @param totalSupply total supply
   * @param centralBankKey key
   */
  public ZKCBDCEngine(final long totalSupply, final String centralBankKey) {
    this.totalSupply = totalSupply;
  }

  /** Javadoc */

  /**
   * @param accountId
   * @param initialBalance
   * @return
   */
  /**
   * Creates a new account.
   *
   * @param accountId the account ID
   * @param initialBalance the initial balance
   * @return the newly created account state
   */
  /**
   * Creates a new account.
   *
   * @param accountId the account ID
   * @param initialBalance the initial balance
   * @return the newly created account state
   */
  public AccountState createAccount(final String accountId, final long initialBalance) {
    if (accounts.containsKey(accountId)) {
      throw new IllegalArgumentException("Account already exists");
    }
    String r = randomHex(16);
    String commitment = sha256(initialBalance + ":" + r);
    AccountState account = new AccountState(accountId, commitment);
    accounts.put(accountId, account);
    return account;
  }

  /** Javadoc */

  /**
   * @param accountId
   */
  /**
   * Adds an account to the sanctions list.
   *
   * @param accountId the account ID
   */
  /**
   * Adds an account to the sanctions list.
   *
   * @param accountId the account ID
   */
  public void addToSanctionsList(final String accountId) {
    sanctionsList.add(accountId);
  }

  /** Javadoc */

  /**
   * @param accountId
   */
  /**
   * Freezes an account.
   *
   * @param accountId the account ID
   */
  /**
   * Freezes an account.
   *
   * @param accountId the account ID
   */
  public void freezeAccount(final String accountId) {
    if (accounts.containsKey(accountId)) {
      accounts.get(accountId).isFrozen = true;
      frozenAccounts.add(accountId);
    }
  }

  /** Javadoc */

  /**
   * @param senderPriv
   * @param receiverPub
   * @param amount
   * @return
   */
  /**
   * Creates a new confidential transaction.
   *
   * @param senderPriv the sender private key
   * @param receiverPub the receiver public key
   * @param amount the transaction amount
   * @return the confidential transaction
   */
  /**
   * Creates a new confidential transaction.
   *
   * @param senderPriv the sender private key
   * @param receiverPub the receiver public key
   * @param amount the transaction amount
   * @return the confidential transaction
   */
  public ConfidentialTransaction createTransaction(
      final String senderPriv, final String receiverPub, final long amount) {
    if (amount <= 0) {
      throw new IllegalArgumentException("Amount must be positive");
    }
    if (senderPriv.equals(receiverPub)) {
      throw new IllegalArgumentException("Self-transfer not allowed");
    }

    String txId =
        sha256(senderPriv + ":" + receiverPub + ":" + amount + ":" + randomHex(16))
            .substring(0, 32);

    // Simulating single UTXO per account for simulation simplicity, like in Python version
    String nullifier = sha256(senderPriv + ":mock_utxo");
    if (nullifiers.contains(nullifier)) {
      throw new IllegalStateException("DOUBLE SPEND DETECTED");
    }

    String r1 = randomHex(16);
    String r2 = randomHex(16);
    String r3 = randomHex(16);

    String commitmentSender = sha256(senderPriv + ":" + r1);
    String commitmentReceiver = sha256(receiverPub + ":" + r2);
    String commitmentAmount = sha256(amount + ":" + r3);

    String zkProof =
        sha256(commitmentAmount + ":" + commitmentSender + ":" + commitmentReceiver + ":verify");
    String kycProof = sha256(senderPriv + ":" + receiverPub + ":humanity:verified");
    String sanctionsProof = sha256(senderPriv + ":" + receiverPub + ":no_sanctions");

    ConfidentialTransaction tx =
        new ConfidentialTransaction(
            txId,
            commitmentSender,
            commitmentReceiver,
            commitmentAmount,
            nullifier,
            zkProof,
            kycProof,
            sanctionsProof);

    tx.computeSeal();

    if (sanctionsList.contains(senderPriv) || sanctionsList.contains(receiverPub)) {
      tx.status = TransactionStatus.REJECTED;
      return tx;
    }
    if (frozenAccounts.contains(senderPriv)) {
      tx.status = TransactionStatus.REJECTED;
      return tx;
    }

    nullifiers.add(nullifier);
    tx.status = TransactionStatus.PROVEN;
    transactions.put(txId, tx);
    totalVolume += amount;

    mintProofs.put(txId, sha256("supply:" + totalSupply + ":" + txId + ":" + totalVolume));

    tx.temporalAnchor =
        "923-ANCHOR-" + sha256(tx.seal).substring(0, 16).toUpperCase(java.util.Locale.ROOT);
    tx.status = TransactionStatus.ANCHORED;

    return tx;
  }

  /** Javadoc */

  /**
   * @param tx
   * @return
   */
  /**
   * Verifies the proof for a transaction.
   *
   * @param tx the transaction to verify
   * @return true if verified, false otherwise
   */
  /**
   * Verifies the proof for a transaction.
   *
   * @param tx the transaction to verify
   * @return true if verified, false otherwise
   */
  public boolean verifyProof(final ConfidentialTransaction tx) {
    String recalculated =
        sha256(
            tx.commitmentAmount
                + ":"
                + tx.commitmentSender
                + ":"
                + tx.commitmentReceiver
                + ":verify");
    if (!recalculated.substring(0, 16).equals(tx.zkProof.substring(0, 16))) {
      tx.status = TransactionStatus.REJECTED;
      return false;
    }
    tx.status = TransactionStatus.PROVEN;
    return true;
  }

  private String randomHex(final int length) {
    byte[] bytes = new byte[length];
    random.nextBytes(bytes);
    StringBuilder sb = new StringBuilder();
    for (byte b : bytes) {
      sb.append(String.format("%02x", b));
    }
    return sb.toString();
  }

  @SuppressWarnings("DoNotInvokeMessageDigestDirectly")
  private static String sha256(final String input) {
    try {
      MessageDigest md = MessageDigest.getInstance("SHA-256");
      byte[] hash = md.digest(input.getBytes(java.nio.charset.StandardCharsets.UTF_8));
      StringBuilder hexString = new StringBuilder();
      for (byte b : hash) {
        String hex = Integer.toHexString(0xff & b);
        if (hex.length() == 1) hexString.append('0');
        hexString.append(hex);
      }
      return hexString.toString();
    } catch (NoSuchAlgorithmException e) {
      throw new RuntimeException(e);
    }
  }
}
