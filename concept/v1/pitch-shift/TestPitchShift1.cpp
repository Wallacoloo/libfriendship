/* We desire smooth pitch changes (e.g. "risers", "pitch bends", etc).
 * One option for implementing this is to break a signal into pieces,
 * where each piece consists of two distinct pitches that are tweened.
 * That is, we start at Pitch#1, then fade it out and fade Pitch#2 in.
 *
 * This file tests the above method to see if the trivial implementation
 * sounds acceptable (it does, for most chunk sizes).
 */

#include <cstdint>
#include <algorithm>
#include <cmath>
#include <cstdio>

#include <sndfile.hh>

#define TWO_PI (2*M_PI)
#define SAMPLE_RATE (44100)
#define LENGTH (SAMPLE_RATE*5)
#define SAMPLE_TO_SEC(i) ((i)*1.0f/(float)SAMPLE_RATE)


// approximate y=sin(at^2+bt) via y = f1(t)*sin((a*t1+b)*t) + f2(t)*sin((a*t2+b)*t)
static void pitchSeg(float *out, int t0, int t1, float a, float b)
{
	// start and end phase
	float w0 = a*t0*t0 + b*t0;
	float w1 = a*t1*t1 + b*t1;
	// y = sin(a*t*t + b*t)
	// a = (t1-t)/(t1-t0) * sin(w0 + fL*(t-t0)) + (t-t0)/(t1-t0) * sin(w0 + fR*(t-t0))
	// y(t0) = a(t0)
	//   sin(a*t0*t0 + b*t0) = sin(w0) .:. w0 = a*t0*t0 + b*t0
	// y(t1) = a(t1)
	//   sin(a*t1*t1 + b*t1) = sin(w1) = sin(w0 + fR*(t1-t0)) .:. w1 = w0 + fR*(t1-t0)
	//   .:. fR = (w1-w0)/(t1-t0)
	// a'(t0) = y'(t0)   [approximate]
	// (2*a*t0+b)*cos(w0) = fL*cos(w0) .:. fL=2*a*t0+b
	float freqR = (w1-w0) / (float)(t1-t0);
	float freqL = 2*a*t0 + b;
	for (int t=t0; t<t1; ++t)
	{
		out[t] = (t1-t)/(float)(t1-t0) * sin(w0 + freqL*(t-t0)) +
		         (t-t0)/(float)(t1-t0) * sin(w0 + freqR*(t-t0));
	}
}


static void approxPitchShift(float *out, int blockSize, float a, float b)
{
	// perform an approximate pitch shift over each group of @blockSize samples.
	// a true pitch shift is when @blockSize=1
	for (int i=0; i<LENGTH; i+=blockSize)
	{
		pitchSeg(out, i, std::min(LENGTH, i+blockSize), a, b);
	}
}



static void convertTo16(float *inp, int16_t *outp)
{
	for (int i=0; i<LENGTH; ++i)
	{
		outp[i] = inp[i]*((1<<15) - 1);
	}
}

static void doOutput(int blockSize, const char *fileName)
{
	float bufferF[LENGTH];
	int16_t buffer16[LENGTH];
	SndfileHandle outFile(fileName, SFM_WRITE, SF_FORMAT_WAV | SF_FORMAT_PCM_16, 1, SAMPLE_RATE);

	approxPitchShift(bufferF, blockSize, 500.0*TWO_PI/SAMPLE_RATE/SAMPLE_RATE, 110.0*TWO_PI/SAMPLE_RATE);

	convertTo16(bufferF, buffer16);

	outFile.write(buffer16, LENGTH);
}

int main(int argc, char** argv)
{
	doOutput(1,    "block1.wav");
	doOutput(4,    "block4.wav");
	doOutput(16,   "block16.wav");
	doOutput(64,   "block64.wav");
	doOutput(128,  "block128.wav");
	doOutput(256,  "block256.wav");
	doOutput(512,  "block512.wav");
	doOutput(1024, "block1024.wav");
	doOutput(2048, "block2048.wav");
	doOutput(4096, "block4096.wav");
	doOutput(16384,"block16384.wav");

	return 0;
}
