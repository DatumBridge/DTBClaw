# Οδηγός Ενημέρωσης και Απεγκατάστασης στο macOS

Αυτή η σελίδα τεκμηριώνει τις υποστηριζόμενες διαδικασίες ενημέρωσης και απεγκατάστασης του OctoClaw στο macOS (OS X).

Τελευταία επαλήθευση: **22 Φεβρουαρίου 2026**.

## 1) Έλεγχος τρέχουσας μεθόδου εγκατάστασης

```bash
which octoclaw
octoclaw --version
```

Τυπικές τοποθεσίες:

- Homebrew: `/opt/homebrew/bin/octoclaw` (Apple Silicon) ή `/usr/local/bin/octoclaw` (Intel)
- Cargo/bootstrap/χειροκίνητη: `~/.cargo/bin/octoclaw`

Αν υπάρχουν και οι δύο, η σειρά `PATH` του shell σας καθορίζει ποια εκτελείται.

## 2) Ενημέρωση στο macOS

### Α) Εγκατάσταση μέσω Homebrew

```bash
brew update
brew upgrade octoclaw
octoclaw --version
```

### Β) Εγκατάσταση μέσω Clone + bootstrap

Από τον τοπικό κλώνο του αποθετηρίου:

```bash
git pull --ff-only
./bootstrap.sh --prefer-prebuilt
octoclaw --version
```

Αν θέλετε ενημέρωση μόνο από πηγαίο κώδικα:

```bash
git pull --ff-only
cargo install --path . --force --locked
octoclaw --version
```

### Γ) Χειροκίνητη εγκατάσταση προκατασκευασμένου binary

Επαναλάβετε τη ροή λήψης/εγκατάστασης με το πιο πρόσφατο αρχείο έκδοσης και επαληθεύστε:

```bash
octoclaw --version
```

## 3) Απεγκατάσταση στο macOS

### Α) Διακοπή και αφαίρεση υπηρεσίας background πρώτα

Αυτό αποτρέπει τη συνέχεια εκτέλεσης του daemon μετά την αφαίρεση του binary.

```bash
octoclaw service stop || true
octoclaw service uninstall || true
```

Αντικείμενα υπηρεσίας που αφαιρούνται από την `service uninstall`:

- `~/Library/LaunchAgents/com.octoclaw.daemon.plist`

### Β) Αφαίρεση binary ανά μέθοδο εγκατάστασης

Homebrew:

```bash
brew uninstall octoclaw
```

Cargo/bootstrap/χειροκίνητη (`~/.cargo/bin/octoclaw`):

```bash
cargo uninstall octoclaw || true
rm -f ~/.cargo/bin/octoclaw
```

### Γ) Προαιρετικά: αφαίρεση τοπικών δεδομένων εκτέλεσης

Εκτελέστε αυτό μόνο αν θέλετε πλήρη εκκαθάριση ρυθμίσεων, προφίλ auth, logs και κατάστασης workspace.

```bash
rm -rf ~/.octoclaw
```

## 4) Επαλήθευση ολοκλήρωσης απεγκατάστασης

```bash
command -v octoclaw || echo "octoclaw binary not found"
pgrep -fl octoclaw || echo "No running octoclaw process"
```

Αν το `pgrep` εξακολουθεί να βρίσκει διεργασία, σταματήστε την χειροκίνητα και ελέγξτε ξανά:

```bash
pkill -f octoclaw
```

## Σχετικά Έγγραφα

- [One-Click Bootstrap](../one-click-bootstrap.md)
- [Αναφορά Εντολών](../commands-reference.md)
- [Αντιμετώπιση Προβλημάτων](../troubleshooting.md)
