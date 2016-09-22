#!/usr/bin/env python3

from math import cos, pi

from numpy.linalg import lstsq as least_squares

def flatten(arr):
    f = []
    for entry in arr:
        if isinstance(entry, list) or isinstance(entry, tuple):
            f.extend(flatten(entry))
        else:
            f.append(entry)
    return f


def get_coeffs():
    # Design a filter of form:
    #h[0] = a0 + a1*c
    #h[1] = a2 + a3*c
    #h[2] = a4 + a5*c
    #...
    #where c is cutoff frequency
    # and a_i are chosen to minimize the least-squares between the achieved filters and an ideal LPF
    # over some range.
    # Note, we would like the filter to have constant delay, and we expect it to be symmetric.
    # Therefore (consider n=3),
    # h[0]*w^2 + h[1] w^1 + h[2] w^0 = H(e^jw) * w^1
    # h[0]*w^1 + h[1]*w^0 + h[2] w^-1 = H(e^jw)
    # In order for this to be real, it's clear that h[i] = h[n-1-i]
    # Thus,
    # h[0]*w^1 + h[1]*w^0 + h[0] w^-1 = H(e^jw)
    # h[0]*cos(w*1)*2 + h[1]*cos(w*0) = H(e^jw)
    # So we build a matrix:
    # [cos(w*0), 2*cos(w*1), 2*cos(w*2), ...] x = [H(e^jw)]
    # [ ...]
    # For several w.
    #
    # Substitute h[0] = a0+a1*c, etc:
    # [cos(w*0), c*cos(w*0), 2*cos(w*1), 2*c*cos(w*1), ...] * [a0, a1, a2, a3, ...]^T = [H(e^jw)]
    # Out of simplicity, remove the 2* factors, and add them in AFTER solving for a2,a3, ...
    delay = 2 # expected delay, in samples
    length = 2*delay + 1 # Length of h[n]. Determines minimum cutoff (i.e. pi/length)
    one_sided_length = delay+1
    freq_spectrum_n = 16 # Number of frequencies to trace.

    A = []
    b = []
    # Try a number of cutoff frequencies
    for cutoff_i in range(freq_spectrum_n+1):
        cutoff = (cutoff_i+0.5)/freq_spectrum_n * pi
        # Test each frequency response at this cutoff.
        for j in range(freq_spectrum_n+1):
            test_freq = j / freq_spectrum_n * pi
            expected_resp = int(test_freq < cutoff)
            # Compute matrix coefficients:
            entries = flatten([(cos(test_freq*i), cutoff*cos(test_freq*i)) for i in range(one_sided_length)])
            A.append(entries)
            b.append(expected_resp)
    print(A,b)
    sol = least_squares(A,b)
    partial_coeffs = sol[0]
    scaled_coeffs = [v/(1 + (i>=2))  for i,v in enumerate(partial_coeffs)]
    return scaled_coeffs

if __name__ == "__main__":
    coeffs = get_coeffs()
    print(coeffs)
