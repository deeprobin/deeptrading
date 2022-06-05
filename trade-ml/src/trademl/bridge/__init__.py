import os
from typing import Callable, Optional
from enum import Enum
import numpy as np
#import pickle
#from envs import TradingEnv
#from agent import DQNAgent
#from utils import get_data, get_scaler, maybe_make_dir
from trademl.ai.environment import Environment1
from trademl.ai.network import train_dddqn, train_dqn, train_ddqn
from trademl.interface import NodeInterface


class NodeState(Enum):
    INITIALIZATION = 1
    TRAINING = 2
    TESTING = 3
    TRADING = 4


class NodeBridge:
    state_change_callback: Optional[Callable[[NodeState], None]] = None
    interface: NodeInterface
    data_dir: str
    episodes: int = 20_000
    batch_size: int = 32

    def __init__(self, interface: NodeInterface, data_dir='data'):
        self.interface = interface
        self.data_dir = data_dir
        if not os.path.exists(data_dir):
            os.makedirs(data_dir)

    def run(self):
        if self.state_change_callback is not None:
            self.state_change_callback(NodeState.INITIALIZATION)
        weights_folder = os.path.join(self.data_dir, 'weights')
        portfolio_folder = os.path.join(self.data_dir, 'portfolio_val')
        if not os.path.exists(weights_folder):
            os.makedirs(weights_folder)
        if not os.path.exists(portfolio_folder):
            os.makedirs(portfolio_folder)

#        data: np.array = np.around(self.get_data_fn())
#        train_data = data[:, :3526]
#        test_data = data[:, 3526:]
#
#        env = TradingEnv(train_data, 20_000)
#        state_size = env.observation_space.shape
#        action_size = env.action_space.n
#        agent = DQNAgent(state_size, action_size)
#        scaler = get_scaler(env)
#
#        portfolio_value = []
#
#        self.state_change_callback(NodeState.TRAINING)
#        timestamp = time.strftime('%Y%m%d%H%M')
#        for e in range(self.episodes):
#            state = env.reset()
#            state = scaler.transform([state])
#            i = 0
#            for time in range(env.n_step):
#                i += 1
#                print(f"{i}/{env.n_step}")
#                action = agent.act(state)
#                next_state, reward, done, info = env.step(action)
#                next_state = scaler.transform([next_state])
#                agent.remember(state, action, reward, next_state, done)
#                state = next_state
#                if done:
#                    print("episode: {}/{}, episode end value: {}".format(e +
#                          1, self.episodes, info['cur_val']))
#                    # append episode end portfolio value
#                    portfolio_value.append(info['cur_val'])
#                    break
#                if len(agent.memory) > self.batch_size:
#                    agent.replay(self.batch_size)
#            if (e + 1) % 10 == 0:  # checkpoint weights
#                agent.save(os.path.join(weights_folder, f"{timestamp}.h5"))
#        # save portfolio value history to disk
#        # with open('portfolio_val/{}-{}.p'.format(timestamp, 'train'), 'wb') as fp:
#        #    pickle.dump(portfolio_value, fp)
#
#        self.state_change_callback(NodeState.TESTING)
        data = self.interface.get_data()
        train = data[:1000]
        test = data[1000:]
        env = Environment1(train)
        print(env.reset())
        for _ in range(3):
            pact = np.random.randint(3)
            print(env.step(pact))

        Q, total_losses, total_rewards = train_dqn(Environment1(train))
        print(
            f"Q: {Q}, total_losses: {total_losses}, total_rewards: {total_rewards}")

        Q, total_losses, total_rewards = train_ddqn(Environment1(train))
        print(
            f"Q: {Q}, total_losses: {total_losses}, total_rewards: {total_rewards}")
        Q, total_losses, total_rewards = train_dddqn(Environment1(train))

        print(
            f"Q: {Q}, total_losses: {total_losses}, total_rewards: {total_rewards}")
