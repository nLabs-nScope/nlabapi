import matplotlib.pyplot as plt
import numpy as np
from nlabapi import LabBench, AnalogSignalPolarity

nlab = LabBench.open_first_available()

nlab.ax_turn_on(1)
nlab.ax_set_amplitude(1, 3.5)
nlab.ax_set_polarity(1, AnalogSignalPolarity.Bipolar)
number_of_samples = 3000
sample_rate = 1000.0

data = nlab.read_all_channels(sample_rate, number_of_samples)
nlab.ax_turn_off(1)
plt.plot(np.arange(number_of_samples)/sample_rate, data[0], label="Ch1")
plt.plot(np.arange(number_of_samples)/sample_rate, data[1], label="Ch2")
plt.plot(np.arange(number_of_samples)/sample_rate, data[2], label="Ch3")
plt.plot(np.arange(number_of_samples)/sample_rate, data[3], label="Ch4")
plt.xlabel("Time (s)")
plt.ylabel("Voltage (V)")
plt.legend()
plt.show()
