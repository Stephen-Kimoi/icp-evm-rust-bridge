import { useEffect, useState } from 'react';
import './App.scss';
import rustLogo from './assets/rust.svg';
import reactLogo from './assets/react.svg';
import ethLogo from './assets/eth.svg';
import icpLogo from './assets/ICP.png';
import { backend } from './declarations/backend';
import { Block } from './declarations/backend/backend.did';
import { ToastContainer, toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import { JsonView, allExpanded, defaultStyles } from 'react-json-view-lite';
import 'react-json-view-lite/dist/index.css';
import { BarLoader } from 'react-spinners';

function App() {
  const [loading, setLoading] = useState(false);
  const [block, setBlock] = useState<Block | undefined>();
  const [canisterEthAddress, setCanisterEthAddress] = useState<string | undefined>();
  const [count, setCount] = useState<number | undefined>();
  const [transactionHashes, setTransactionHashes] = useState<string[]>([]);

  const fetchCanisterEthAddress = async () => {
    try {
      setLoading(true);
      const address = await backend.get_canister_eth_address();
      setCanisterEthAddress(address);
      toast.success('Canister Ethereum address fetched successfully!');
    } catch (err) {
      console.error(err);
      toast.error('Failed to fetch the canister Ethereum address.');
    } finally {
      setLoading(false);
    }
  };

  const fetchCount = async () => {
    try {
      setLoading(true);
      const result = await backend.get_count();
      if ('Ok' in result) {
        setCount(Number(result.Ok));
        toast.success('Count fetched successfully!');
      } else if ('Err' in result) {
        throw new Error(result.Err);
      } else {
        throw new Error('Unexpected response format');
      }
    } catch (err) {
      console.error(err);
      toast.error('Failed to fetch the count: ' + (err instanceof Error ? err.message : String(err)));
    } finally {
      setLoading(false);
    }
  };

  const increaseCount = async () => {
    try {
      setLoading(true);
      await backend.call_increase_count();
      toast.success('Count increased successfully!');
      fetchCount(); // Refresh the count after increasing
    } catch (err) {
      console.error(err);
      toast.error('Failed to increase the count.');
    } finally {
      setLoading(false);
    }
  };

  const decreaseCount = async () => {
    try {
      setLoading(true);
      await backend.call_decrease_count();
      toast.success('Count decreased successfully!');
      fetchCount(); // Refresh the count after decreasing
    } catch (err) {
      console.error(err);
      toast.error('Failed to decrease the count.');
    } finally {
      setLoading(false);
    }
  };

  const fetchTransactionHashes = async () => {
    try {
      const hashes = await backend.get_stored_transaction_hashes();
      console.log("Hashes are: ", hashes); 
      setTransactionHashes(hashes);
    } catch (err) {
      console.error(err);
      toast.error('Failed to fetch transaction hashes.');
    } 
  };

  useEffect(() => {
    fetchTransactionHashes(); 
  })

  return (
    <div className="App">

      <div>
        <a
          href="https://github.com/internet-computer-protocol/evm-rpc-canister#readme"
          target="_blank"
        >
          <img src={ethLogo} className="logo ethereum" alt="Ethereum logo" />
        </a>
        <a
          href="https://internetcomputer.org/docs/current/developer-docs/backend/rust/"
          target="_blank"
        >
          <span className="logo-stack">
            <img src={icpLogo} className="logo rust" alt="Rust logo" />
          </span>
        </a>
      </div>

      <h1 style={{ paddingLeft: 36 }}>Chain fusion: ICP + EVM Starter</h1>
      
      <p className="read-the-docs">
        Interact with smart contract deployed on Ethereum using ICP's Rust canister 
      </p>

      <div className="card" style={{ opacity: loading ? 0.5 : 1 }}>

        <h2>Read Functionality</h2>
        <div className="button-group">
          <button className="fancy-button" onClick={fetchCanisterEthAddress}>
            Get Canister ETH Address
          </button>
          <button className="fancy-button" onClick={fetchCount}>
            Get Count
          </button>
        </div>

        <h2>Write Functionality</h2>
        <div className="button-group">
          <button className="fancy-button" onClick={increaseCount}>
            Increase Count
          </button>
          <button className="fancy-button" onClick={decreaseCount}>
            Decrease Count
          </button>
        </div>

        {loading && (
          <div className="loader">
            <BarLoader color="#36d7b7" width={200} />
          </div>
        )}
        {!!block && (
          <pre className="json-view">
            <h3>Latest Ethereum Block:</h3>
            <JsonView
              data={block}
              shouldExpandNode={allExpanded}
              style={{ ...defaultStyles, container: '' }}
            />
          </pre>
        )}
        {!!canisterEthAddress && (
          <pre className="result">
            <h3>Canister Ethereum Address:</h3>
            <p>{canisterEthAddress}</p>
          </pre>
        )}
        {count !== undefined && (
          <pre className="result">
            <h3>Count:</h3>
            <p>{count}</p>
          </pre>
        )}

        {transactionHashes.length > 0 && (
          <div className="transaction-hashes">
            <h3>Transaction Hashes:</h3>
            <ul>
              {transactionHashes.map((hash, index) => (
                <li key={index}>
                  <a
                    href={`https://sepolia.etherscan.io/tx/${hash}`}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {hash}
                  </a>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      <ToastContainer position="bottom-right" />
    </div>
  );
}

export default App;

