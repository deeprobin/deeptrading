import math
import random
import numpy as np
import pandas as pd
import tensorflow as tf
import matplotlib.pyplot as plt
import pandas_datareader as data_reader
import keras
from tqdm import tqdm_notebook, tqdm
from collections import deque


class StockPredictor:
    def __init__(self, state_size, action_space=3, model_name="AITrader"):
        self.state_size = state_size
        self.action_space = action_space
        self.memory = deque(2000)
        self.inventory = []
        self.model_name = model_name

        self.gamma = 0.95
        self.epsilon = 1.0
        self.epsilon_final = 0.01
        self.epsilon_decay = 0.995

    def model_builder(self):
        model = keras.models.Sequential()

        model.add(tf.layers.Dense(
            units=32, activation='relu', input_dim=self.state_size))
        model.add(tf.layers.Dense(units=64, activation='relu'))
        model.add(tf.layers.Dense(units=128, activation='relu'))
        model.add(tf.layers.Dense(units=self.action_space, activation='linear'))
