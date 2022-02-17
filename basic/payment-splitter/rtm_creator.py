from typing import List, Union, Dict
import subprocess
import random
import re
import os

def os_command(*args: Union[str, int, float]) -> str:
    """ 
    A function which is used to run an OS command on the command prompt
    and then get the response back to us. Accepts a string or a list of
    strings which are concatenated later.

    # Arguments

    - `args: List[Union[str, int, float]]` - A list of arguments which 
    makes up the command that we wish to run

    # Returns

    - `str` - A string of the command result
    """
    
    stdout: bytes
    stderr: bytes
    stdout, stderr = subprocess.Popen(
        args = " ".join(map(str, args)),
        shell = True,
        stdout = subprocess.PIPE,
        stderr = subprocess.PIPE
    ).communicate()

    return max([stdout, stderr], key = lambda x: len(x)).decode().strip()

def clean_manifest_content(string: str) -> str:
    """ Cleans the content of an un-clean transaction manifest file """

    # Getting the lines contained in the manifest file
    lines: List[str] = string.split('\n')

    # un-indenting all of the lines as the multiline python strings are typically indented
    lines = list(map(lambda x: x.strip(), lines))

    # Removing the lines containing comments (i.e. starting with #)
    lines = list(filter(lambda x: not x.startswith("#"), lines))

    return "\n".join(lines).strip()

class Account():
    """ Defines a simulator account created through resim """

    def __init__(self, address: str, public_key: str) -> None:
        self.address: str = address
        self.public_key: str = public_key

    def __str__(self) -> str:
        return f"Account(address = {self.address}, public_key = {self.public_key})"

    def __repr__(self) -> str:
        return str(self)

    @classmethod
    def new(cls) -> 'Account':
        """ Creates a new account through resim """
        response: str = os_command('resim', 'new-account')
        pub_key: str = re.findall(r'Public key: (\w+)', response)[0]
        address: str = re.findall(r'Account address: (\w+)', response)[0]

        return cls(address, pub_key)
    
    def set_default(self) -> None:
        """ Sets this account as the default account """
        os_command('resim', 'set-default-account', self.address, self.public_key)
 
def main() -> None:
    # Constants which our program will need
    RADIX_TOKEN: str = "030000000000000000000000000000000000000000000000000004"
    CURRENT_SCRIPT_DIR: str = os.path.dirname(os.path.realpath(__file__))
    MANIFEST_DIR: str = os.path.join(CURRENT_SCRIPT_DIR, 'transactions')

    # Resetting the simulator for this run
    os_command('resim', 'reset')
    
    # Creating a number of accounts to use for the testing of this component and then setting the
    # first created account as the default account
    accounts: List[Account] = [Account.new() for _ in range(4)]
    accounts[0].set_default()

    print(f"Created {len(accounts)} accounts:")
    for acc in accounts:
        print(f"\t{acc}")

    # Publishing the package and then deploying the component from this package
    response: str = os_command('resim', 'publish', '.')
    package: str = re.findall(r'Package: (\w+)', response)[0]
    print("Published the package:", package)

    response: str = os_command('resim', 'call-function', package, 'PaymentSplitter', 'new')
    result: List[str] = re.findall(r'ResourceDef: (\w+)', response)
    adm, iadm, shb = result
    component: str = re.findall(r'Component: (\w+)', response)[0]
    print('Instantiated the component:')
    print('\tComponent:', component)
    print('\tAdmin Badge:', adm)
    print('\tInternal Admin Badge:', iadm)
    print('\tShareholders Badge:', shb)
    
    # A dictionary of string and multiline strings where the manifest files are defined. The key in
    # the dictionary is the name of the file that these manifest files should be saved under while
    # the value is the un-cleaned content of the files
    manifest_file_content_mapping: Dict[str, str] = {
        "component_creation": f"""
        
        # This is a transaction manifest file used for the creation of the component from an already
        # published package.
        CALL_FUNCTION Address("{package}") "PaymentSplitter" "new";
        CALL_METHOD_WITH_ALL_RESOURCES Address("{component}") "deposit_batch";
        
        """,

        "adding_shareholders": f"""
        
        # In this manifest transaction file we define the transaction that the admin needs to make
        # in order for them to add shareholders to their payment splitter. 

        # The first thing that the admin needs to do is to withdraw their admin badge from their
        # account and then put it in the transaction worktop. From there, we may create a bucket 
        # this admin badge and eventually a BucketRef. We need a BucketRef as all badges are passed
        # to functions and methods as bucket refs.
        CALL_METHOD Address("{accounts[0].address}") "withdraw" Decimal("1.0") Address("{adm}")  BucketRef(1u32);
        TAKE_FROM_WORKTOP Decimal("1.0") Address("{adm}") Bucket("admin_badge_bucket");
        CREATE_BUCKET_REF Bucket("admin_badge_bucket") BucketRef("admin_badge_bucket_ref0");

        # The bucket references to the bucket containing the badge will be dropped once they're used
        # by one of the methods or function. This means that we need to have as many bucket refs to
        # the badge as function/method calls that utilize and use the badge
        CLONE_BUCKET_REF BucketRef("admin_badge_bucket_ref0") BucketRef("admin_badge_bucket_ref1");
        CLONE_BUCKET_REF BucketRef("admin_badge_bucket_ref0") BucketRef("admin_badge_bucket_ref2");
        CLONE_BUCKET_REF BucketRef("admin_badge_bucket_ref0") BucketRef("admin_badge_bucket_ref3");
        
        # Now we have a bucket and a bucket ref of the admin badge which is one of the things that 
        # we needed in order to be able to call the `add_shareholder` method. We may now call this
        # method for all four of the shareholders that we wish to add.
        CALL_METHOD Address("{component}") "add_shareholder" Address("{accounts[0].address}") Decimal("{float(random.randint(50, 150)):.2f}") BucketRef("admin_badge_bucket_ref0");
        CALL_METHOD Address("{component}") "add_shareholder" Address("{accounts[1].address}") Decimal("{float(random.randint(50, 150)):.2f}") BucketRef("admin_badge_bucket_ref1");
        CALL_METHOD Address("{component}") "add_shareholder" Address("{accounts[2].address}") Decimal("{float(random.randint(50, 150)):.2f}") BucketRef("admin_badge_bucket_ref2");
        CALL_METHOD Address("{component}") "add_shareholder" Address("{accounts[3].address}") Decimal("{float(random.randint(50, 150)):.2f}") BucketRef("admin_badge_bucket_ref3");

        # At this point of time, we no longer need the bucket containing the admin badge as we wont
        # be using it any longer. Therefore, let's proceed to deposit it back into the account of 
        # the admin. 
        CALL_METHOD Address("{accounts[0].address}") "deposit" Bucket("admin_badge_bucket");

        # With the admin badge deposited back into the admin's account, the only thing that we have 
        # left on the transaction worktop is the shareholder's NFTs. We may now get theft NFTs one 
        # by one and deposit them into the appropriate accounts
        TAKE_NON_FUNGIBLES_FROM_WORKTOP TreeSet<NonFungibleKey>(NonFungibleKey("{0:032x}")) Address("{shb}") Bucket("badge_0");
        CALL_METHOD Address("{accounts[0].address}") "deposit" Bucket("badge_0");

        TAKE_NON_FUNGIBLES_FROM_WORKTOP TreeSet<NonFungibleKey>(NonFungibleKey("{1:032x}")) Address("{shb}") Bucket("badge_1");
        CALL_METHOD Address("{accounts[1].address}") "deposit" Bucket("badge_1");

        TAKE_NON_FUNGIBLES_FROM_WORKTOP TreeSet<NonFungibleKey>(NonFungibleKey("{2:032x}")) Address("{shb}") Bucket("badge_2");
        CALL_METHOD Address("{accounts[2].address}") "deposit" Bucket("badge_2");

        TAKE_NON_FUNGIBLES_FROM_WORKTOP TreeSet<NonFungibleKey>(NonFungibleKey("{3:032x}")) Address("{shb}") Bucket("badge_3");
        CALL_METHOD Address("{accounts[3].address}") "deposit" Bucket("badge_3");

        """,

        "funding_the_splitter": f"""
        
        # In order to test to ensure that the splitter does indeed work, we need to fund the splitter
        # with funds which we later withdraw using other accounts. 

        # Withdrawing XRD from the main account and putting it in a bucket so we can latter call the
        # `deposit_xed` method on the payment splitter to give it the XRD.
        CALL_METHOD Address("{accounts[0].address}") "withdraw" Decimal("{90_000:.2f}") Address("{RADIX_TOKEN}")  BucketRef(1u32);
        TAKE_FROM_WORKTOP Decimal("{90_000:.2f}") Address("{RADIX_TOKEN}") Bucket("xrd_bucket");
        CALL_METHOD Address("{component}") "deposit_xrd" Bucket("xrd_bucket");

        """,

        "withdrawing_owed_amount": f"""
        
        # Now we finally have some funds that the payment splitter owes to us! We can now withdraw
        # these funds from our second account!
        
        # The first thing we need to do is to get the shareholder NFT that we own so that we can 
        # present it as a badge when we ask the payment splitter to give us our funds. To do that
        # we will call the `withdraw` method on the account component
        CALL_METHOD Address("{accounts[1].address}") "withdraw" Decimal("1.0") Address("{shb}") BucketRef(1u32);
        TAKE_FROM_WORKTOP Decimal("1.0") Address("{shb}") Bucket("shareholder_badge_bucket");
        CREATE_BUCKET_REF Bucket("shareholder_badge_bucket") BucketRef("shareholder_badge_bucket_ref");

        # Calling the `withdraw_xrd` method on the component
        CALL_METHOD Address("{component}") "withdraw_xrd" BucketRef("shareholder_badge_bucket_ref");
        
        # At this point of time, we have two things in our transaction worktop: the shareholders
        # badge that we used to withdraw to funds, and the XRD that we just withdrew. We may now 
        # deposit all of the currently available resources on the worktop to the account
        CALL_METHOD_WITH_ALL_RESOURCES Address("{accounts[1].address}") "deposit_batch";

        """

    }

    for file_name, file_content in manifest_file_content_mapping.items():
        with open(os.path.join(MANIFEST_DIR, f"{file_name}.rtm"), 'w') as file:
            file.write(clean_manifest_content(file_content))

if __name__ == "__main__":
    main()