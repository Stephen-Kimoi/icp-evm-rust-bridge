import { useState } from 'react';
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

  const fetchBlock = async () => {
    try {
      setLoading(true);
      const block = await backend.get_latest_ethereum_block();
      setBlock(block);
      toast.success('Latest Ethereum block fetched successfully!');
    } catch (err) {
      console.error(err);
      toast.error('Failed to fetch the latest Ethereum block.');
    } finally {
      setLoading(false);
    }
  };

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
      const count = await backend.get_count();
      setCount(Number(count));
      toast.success('Count fetched successfully!');
    } catch (err) {
      console.error(err);
      toast.error('Failed to fetch the count.');
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
      </div>

      <ToastContainer position="bottom-right" />
    </div>
  );
}

export default App;

