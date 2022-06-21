#!/bin/bash

mkdir -p manifests

scrypto build && \
resim reset

export owner_account=$(resim new-account)
export owner=$(echo $owner_account | grep "Account component address: [a-z0-9]{54}" -oE | cut -d: -f2 | xargs)
export owner_private_key=$(echo $owner_account | grep "Private key: [a-z0-9]{54}" -oE | cut -d: -f2 | xargs)

export package=$(resim publish target/wasm32-unknown-unknown/release/minimal_e_commerce.wasm | grep "New Package: [a-z0-9]{54}" -oE | cut -d: -f2 | xargs)
export xrd=$(resim show $owner | grep XRD | cut -d: -f3 | cut -d, -f1 | xargs)

cat <<EOT > manifests/create_catalog.txm
CALL_FUNCTION PackageAddress("$package") "Catalog" "new";
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress("$owner") "deposit_batch";
EOT

export catalog=$(resim run manifests/create_catalog.txm | grep "Component:" | cut -d: -f2 | xargs)

export owner_badge=$(resim show $owner | grep "Owner of catalog" | cut -d: -f3 | cut -d, -f1 | xargs)

cat <<EOT > manifests/become_minter.txm
CALL_METHOD ComponentAddress("$owner") "create_proof_by_amount" Decimal("1.0") ResourceAddress("$owner_badge");

CALL_METHOD ComponentAddress("$catalog") "become_minter";
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress("$owner") "deposit_batch";
EOT

resim run manifests/become_minter.txm > /dev/null

export reference_minter_badge=$(resim show $owner | grep "Reference minter" | cut -d: -f3 | cut -d, -f1 | xargs)

cat <<EOT > manifests/add_reference.txm
CALL_METHOD ComponentAddress("$owner") "create_proof_by_amount" Decimal("1.0") ResourceAddress("$reference_minter_badge");

CALL_METHOD ComponentAddress("$catalog") "register_reference" "Black ink 100 mL" Decimal("150.0");
EOT

export new_reference=$(resim run manifests/add_reference.txm | grep 'Instruction Outputs:' -A 2 | tail -n1 | cut -d' ' -f2 | xargs)

cat <<EOT > manifests/add_stock_to_reference.txm
CALL_METHOD ComponentAddress("$owner") "create_proof_by_amount" Decimal("1.0") ResourceAddress("$reference_minter_badge");

CALL_METHOD ComponentAddress("$catalog") "add_stock_to_reference" ${new_reference} 5u64;
EOT

resim run manifests/add_stock_to_reference.txm > /dev/null

cat <<EOT > manifests/purchase_article.txm
CALL_METHOD ComponentAddress("$owner") "withdraw_by_amount" Decimal("500.0") ResourceAddress("$xrd");
TAKE_FROM_WORKTOP ResourceAddress("$xrd") Bucket("bidding_bucket");

CALL_METHOD ComponentAddress("$catalog") "purchase_article" ${new_reference} 2u64 Bucket("bidding_bucket");
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress("$owner") "deposit_batch";
EOT

resim run manifests/purchase_article.txm > /dev/null

cat <<EOT > manifests/withdraw.txm
CALL_METHOD ComponentAddress("$owner") "create_proof_by_amount" Decimal("1.0") ResourceAddress("$owner_badge");

CALL_METHOD ComponentAddress("$catalog") "withdraw";
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress("$owner") "deposit_batch";
EOT

echo "======================================"
echo "========= BEFORE WITHDRAWING ========="
echo "=========      CATALOG       ========="
echo "=========                    ========="
resim show $catalog
echo "=========                    ========="
echo "=========       OWNER        ========="
echo "=========                    ========="
resim show $owner
echo "=========                    ========="
echo "========= BEFORE WITHDRAWING ========="
echo "======================================"

resim run manifests/withdraw.txm > /dev/null

echo "======================================"
echo "========= AFTER  WITHDRAWING ========="
echo "=========      CATALOG       ========="
echo "=========                    ========="
resim show $catalog
echo "=========                    ========="
echo "=========       OWNER        ========="
echo "=========                    ========="
resim show $owner
echo "=========                    ========="
echo "========= AFTER  WITHDRAWING ========="
echo "======================================"